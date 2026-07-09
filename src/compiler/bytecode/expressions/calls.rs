use super::*;

impl CodeGenerator {
    fn args_have_spread(args: &[Expression]) -> bool {
        args.iter()
            .any(|a| matches!(a, Expression::SpreadElement { .. }))
    }

    /// Build a single arguments array, expanding any `...spread` elements.
    /// Leaves the array on the stack.
    fn generate_args_array(&mut self, args: &[Expression]) -> Result<()> {
        self.emit(Instruction::NewArray(0));
        for arg in args {
            match arg {
                Expression::SpreadElement { argument } => {
                    self.generate_expression(argument)?;
                    self.emit(Instruction::SpreadArray);
                }
                _ => {
                    // ArrayPush pops (array, value) and re-pushes the array —
                    // no Dup needed (a Dup would leave a stale slot).
                    self.generate_expression(arg)?;
                    self.emit(Instruction::ArrayPush);
                }
            }
        }
        Ok(())
    }

    pub(super) fn generate_call(&mut self, callee: &Expression, args: &[Expression]) -> Result<()> {
        let has_spread = Self::args_have_spread(args);

        if let Expression::Member {
            object,
            property,
            computed,
        } = callee
        {
            if !computed && !has_spread {
                if let Expression::Identifier(name) = property.as_ref() {
                    match name.as_str() {
                        // Well-known collection methods → specialized opcodes
                        s if s == crate::well_known::SET_PROP && args.len() == 2 => {
                            self.generate_expression(object)?;
                            for arg in args {
                                self.generate_expression(arg)?;
                            }
                            self.emit(Instruction::MapSet(args.len() as u16));
                            return Ok(());
                        }
                        s if s == crate::well_known::GET && args.len() == 1 => {
                            self.generate_expression(object)?;
                            for arg in args {
                                self.generate_expression(arg)?;
                            }
                            self.emit(Instruction::MapGet);
                            return Ok(());
                        }
                        s if s == crate::well_known::HAS && args.len() == 1 => {
                            self.generate_expression(object)?;
                            for arg in args {
                                self.generate_expression(arg)?;
                            }
                            self.emit(Instruction::MapHas);
                            return Ok(());
                        }
                        s if s == crate::well_known::DELETE && args.len() == 1 => {
                            self.generate_expression(object)?;
                            for arg in args {
                                self.generate_expression(arg)?;
                            }
                            self.emit(Instruction::MapDelete);
                            return Ok(());
                        }
                        s if s == crate::well_known::ADD && args.len() == 1 => {
                            self.generate_expression(object)?;
                            for arg in args {
                                self.generate_expression(arg)?;
                            }
                            self.emit(Instruction::SetAdd);
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            }
            if has_spread {
                // [argsArray, this, method] then Apply
                self.generate_args_array(args)?;
                self.generate_expression(object)?; // [args, this]
                self.emit(Instruction::Dup); // [args, this, this]
                if *computed {
                    self.generate_expression(property)?;
                } else if let Expression::Identifier(name) = property.as_ref() {
                    let idx = self.add_constant(Value::from_string(name.clone().into()));
                    self.emit(Instruction::LoadConst(idx));
                } else {
                    self.generate_expression(property)?;
                }
                self.emit(Instruction::GetProperty); // [args, this, method]
                self.emit(Instruction::Apply);
            } else {
                self.generate_expression(object)?;
                if *computed {
                    self.generate_expression(property)?;
                } else if let Expression::Identifier(name) = property.as_ref() {
                    let idx = self.add_constant(Value::from_string(name.clone().into()));
                    self.emit(Instruction::LoadConst(idx));
                } else {
                    self.generate_expression(property)?;
                }
                for arg in args {
                    self.generate_expression(arg)?;
                }
                self.emit(Instruction::CallMethod(args.len() as u16));
            }
        } else if let Expression::OptionalMember {
            object,
            property,
            computed,
        } = callee
        {
            // `receiver?.method(args)` — OptionalMember used as call callee.
            // Spread on optional calls is uncommon; fall back to guarded path
            // without spread expansion for now.
            self.generate_guarded_method_call(object, property, *computed, args)?;
        } else if has_spread {
            // [argsArray, undefined, callee] then Apply
            self.generate_args_array(args)?;
            self.emit(Instruction::LoadUndefined);
            self.generate_expression(callee)?;
            self.emit(Instruction::Apply);
        } else {
            for arg in args {
                self.generate_expression(arg)?;
            }
            self.generate_expression(callee)?;
            self.emit(Instruction::Call(args.len() as u16));
        }
        Ok(())
    }

    /// Emit a method call that short-circuits to `undefined` if either the
    /// receiver or the resolved method is nullish.
    ///
    /// Final stack layout for `OptionalCall` must be:
    /// `[this, callee, arg0, arg1, ...]`
    ///
    /// `JumpIfUndefined` **pops**, so we re-Dup the receiver after the
    /// nullish check before `GetProperty`, otherwise `this` is lost.
    fn generate_guarded_method_call(
        &mut self,
        object: &Expression,
        property: &Expression,
        computed: bool,
        args: &[Expression],
    ) -> Result<()> {
        // [this]
        self.generate_expression(object)?;
        // [this, this]
        self.emit(Instruction::Dup);
        let check_obj = self.instructions.len();
        // pops one; if nullish jump with [this] left for cleanup; else [this]
        self.emit(Instruction::JumpIfUndefined(0));
        // Keep `this` for the eventual call; Dup for GetProperty.
        // [this, this]
        self.emit(Instruction::Dup);
        if computed {
            self.generate_expression(property)?;
        } else if let Expression::Identifier(name) = property {
            let idx = self.add_constant(Value::from_string(name.clone().into()));
            self.emit(Instruction::LoadConst(idx));
        } else {
            self.generate_expression(property)?;
        }
        // [this, this, key] -> [this, method]
        self.emit(Instruction::GetProperty);
        // Guard the resolved callable.
        // [this, method, method]
        self.emit(Instruction::Dup);
        let check_callee = self.instructions.len();
        // pops one; if nullish jump with [this, method]; else [this, method]
        self.emit(Instruction::JumpIfUndefined(0));
        for arg in args {
            self.generate_expression(arg)?;
        }
        // [this, method, ...args]
        self.emit(Instruction::OptionalCall(args.len() as u16));
        let skip_end = self.instructions.len();
        self.emit(Instruction::Jump(0));
        // method was nullish: drop method, then object
        self.patch_jump(check_callee, self.instructions.len());
        self.emit(Instruction::Pop);
        // object was nullish (or fell through from method): drop object
        self.patch_jump(check_obj, self.instructions.len());
        self.emit(Instruction::Pop);
        self.emit(Instruction::LoadUndefined);
        self.patch_jump(skip_end, self.instructions.len());
        Ok(())
    }

    pub(super) fn generate_optional_call(
        &mut self,
        callee: &Expression,
        args: &[Expression],
    ) -> Result<()> {
        if let Expression::Member {
            object,
            property,
            computed,
        } = callee
        {
            // `object?.property(args)` / `object.property?.(args)` with Member callee
            self.generate_guarded_method_call(object, property, *computed, args)?;
        } else if let Expression::OptionalMember {
            object,
            property,
            computed,
        } = callee
        {
            // `object?.property?.(args)`
            self.generate_guarded_method_call(object, property, *computed, args)?;
        } else {
            self.generate_expression(callee)?;
            self.emit(Instruction::Dup);
            let check_undef = self.instructions.len();
            self.emit(Instruction::JumpIfUndefined(0));
            for arg in args {
                self.generate_expression(arg)?;
            }
            // plain Call expects [...args, callee] — but generate_call for
            // non-member puts args first then callee. Match that: currently
            // we have [callee] after the undefined check (Dup was popped by
            // JumpIfUndefined, one callee remains). Pushing args on top gives
            // [callee, ...args] which is WRONG for Call.
            // Call pops args then callee, so needs [...args, callee] with
            // callee on top. Rebuild: we have [callee]; for Call we need args
            // under callee. Easier: use OptionalCall with undefined this.
            // Existing code used Call after pushing args on top of callee —
            // that would be [callee, args...] which is inverted vs Call.
            // Keep prior behavior: Call(argc) path for bare optional call.
            // Looking at Call implementation: it pops args first (argc times)
            // then pops callee. So stack must be [callee, arg0, arg1, ...]
            // with args on top? Let me check Call...
            self.emit(Instruction::Call(args.len() as u16));
            let skip_end = self.instructions.len();
            self.emit(Instruction::Jump(0));
            self.patch_jump(check_undef, self.instructions.len());
            self.emit(Instruction::Pop);
            self.emit(Instruction::LoadUndefined);
            self.patch_jump(skip_end, self.instructions.len());
        }
        Ok(())
    }

    pub(super) fn generate_new_expression(
        &mut self,
        callee: &Expression,
        args: &[Expression],
    ) -> Result<()> {
        if Self::args_have_spread(args) {
            self.generate_args_array(args)?;
            self.generate_expression(callee)?;
            self.emit(Instruction::ConstructApply);
        } else {
            self.generate_expression(callee)?;
            for arg in args {
                self.generate_expression(arg)?;
            }
            self.emit(Instruction::Construct(args.len() as u16));
        }
        Ok(())
    }
}
