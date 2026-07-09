use super::*;

impl CodeGenerator {
    pub(super) fn generate_call(&mut self, callee: &Expression, args: &[Expression]) -> Result<()> {
        if let Expression::Member {
            object,
            property,
            computed,
        } = callee
        {
            if !computed {
                if let Expression::Identifier(name) = property.as_ref() {
                    match name.as_str() {
                        "set" if args.len() == 2 => {
                            self.generate_expression(object)?;
                            for arg in args {
                                self.generate_expression(arg)?;
                            }
                            self.emit(Instruction::MapSet(args.len() as u16));
                            return Ok(());
                        }
                        "get" if args.len() == 1 => {
                            self.generate_expression(object)?;
                            for arg in args {
                                self.generate_expression(arg)?;
                            }
                            self.emit(Instruction::MapGet);
                            return Ok(());
                        }
                        "has" if args.len() == 1 => {
                            self.generate_expression(object)?;
                            for arg in args {
                                self.generate_expression(arg)?;
                            }
                            self.emit(Instruction::MapHas);
                            return Ok(());
                        }
                        "delete" if args.len() == 1 => {
                            self.generate_expression(object)?;
                            for arg in args {
                                self.generate_expression(arg)?;
                            }
                            self.emit(Instruction::MapDelete);
                            return Ok(());
                        }
                        "add" if args.len() == 1 => {
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
            self.generate_expression(object)?;
            if *computed {
                self.generate_expression(property)?;
            } else if let Expression::Identifier(name) = property.as_ref() {
                let idx = self.add_constant(Value::String(name.clone()));
                self.emit(Instruction::LoadConst(idx));
            } else {
                self.generate_expression(property)?;
            }
            for arg in args {
                self.generate_expression(arg)?;
            }
            self.emit(Instruction::CallMethod(args.len() as u16));
        } else if let Expression::OptionalMember {
            object,
            property,
            computed,
        } = callee
        {
            // Same as the Member branch: guard the resolved member value, not
            // just the object, then call with `OptionalCall`.
            self.generate_expression(object)?; // object (this)
            self.emit(Instruction::Dup);
            let check_obj = self.instructions.len();
            self.emit(Instruction::JumpIfUndefined(0));
            if *computed {
                self.generate_expression(property)?;
            } else if let Expression::Identifier(name) = property.as_ref() {
                let idx = self.add_constant(Value::String(name.clone()));
                self.emit(Instruction::LoadConst(idx));
            } else {
                self.generate_expression(property)?;
            }
            self.emit(Instruction::GetProperty); // object, method
            self.emit(Instruction::Dup);
            let check_callee = self.instructions.len();
            self.emit(Instruction::JumpIfUndefined(0));
            for arg in args {
                self.generate_expression(arg)?;
            }
            // Stack for OptionalCall: args..., this=object, callee=method
            self.emit(Instruction::OptionalCall(args.len() as u16));
            let skip_end = self.instructions.len();
            self.emit(Instruction::Jump(0));
            self.patch_jump(check_callee, self.instructions.len());
            self.emit(Instruction::Pop);
            self.patch_jump(check_obj, self.instructions.len());
            self.emit(Instruction::Pop);
            self.emit(Instruction::LoadUndefined);
            self.patch_jump(skip_end, self.instructions.len());
        } else {
            for arg in args {
                self.generate_expression(arg)?;
            }
            self.generate_expression(callee)?;
            self.emit(Instruction::Call(args.len() as u16));
        }
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
            // Resolve `object[property]` (the callable) and guard it. Unlike a
            // plain optional member access, here we must also support calling
            // the resolved value, so we keep `object` on the stack as `this`
            // and the resolved member as the callee.
            self.generate_expression(object)?; // object (this)
            self.emit(Instruction::Dup);
            let check_obj = self.instructions.len();
            self.emit(Instruction::JumpIfUndefined(0));
            if *computed {
                self.generate_expression(property)?;
            } else if let Expression::Identifier(name) = property.as_ref() {
                let idx = self.add_constant(Value::String(name.clone()));
                self.emit(Instruction::LoadConst(idx));
            } else {
                self.generate_expression(property)?;
            }
            // Stack: object, object, key -> get_property -> object, method
            self.emit(Instruction::GetProperty);
            // Guard the resolved callable, not just the object.
            self.emit(Instruction::Dup);
            let check_callee = self.instructions.len();
            self.emit(Instruction::JumpIfUndefined(0));
            for arg in args {
                self.generate_expression(arg)?;
            }
            // Stack for OptionalCall: args..., this=object, callee=method
            self.emit(Instruction::OptionalCall(args.len() as u16));
            let skip_end = self.instructions.len();
            self.emit(Instruction::Jump(0));
            self.patch_jump(check_callee, self.instructions.len());
            // callee was undefined: drop it, leave object on stack to be popped
            self.emit(Instruction::Pop);
            self.patch_jump(check_obj, self.instructions.len());
            // object was undefined (or callee was): drop object, push undefined
            self.emit(Instruction::Pop);
            self.emit(Instruction::LoadUndefined);
            self.patch_jump(skip_end, self.instructions.len());
        } else if let Expression::OptionalMember {
            object,
            property,
            computed,
        } = callee
        {
            // Guard the resolved member value (not just the object) before
            // calling, preserving `object` as `this`.
            self.generate_expression(object)?; // object (this)
            self.emit(Instruction::Dup);
            let check_obj = self.instructions.len();
            self.emit(Instruction::JumpIfUndefined(0));
            if *computed {
                self.generate_expression(property)?;
            } else if let Expression::Identifier(name) = property.as_ref() {
                let idx = self.add_constant(Value::String(name.clone()));
                self.emit(Instruction::LoadConst(idx));
            } else {
                self.generate_expression(property)?;
            }
            self.emit(Instruction::GetProperty); // object, method
            self.emit(Instruction::Dup);
            let check_callee = self.instructions.len();
            self.emit(Instruction::JumpIfUndefined(0));
            for arg in args {
                self.generate_expression(arg)?;
            }
            // Stack for OptionalCall: args..., this=object, callee=method
            self.emit(Instruction::OptionalCall(args.len() as u16));
            let skip_end = self.instructions.len();
            self.emit(Instruction::Jump(0));
            self.patch_jump(check_callee, self.instructions.len());
            self.emit(Instruction::Pop); // drop method
            self.patch_jump(check_obj, self.instructions.len());
            self.emit(Instruction::Pop); // drop object
            self.emit(Instruction::LoadUndefined);
            self.patch_jump(skip_end, self.instructions.len());
        } else {
            self.generate_expression(callee)?;
            self.emit(Instruction::Dup);
            let check_undef = self.instructions.len();
            self.emit(Instruction::JumpIfUndefined(0));
            for arg in args {
                self.generate_expression(arg)?;
            }
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
        self.generate_expression(callee)?;
        for arg in args {
            self.generate_expression(arg)?;
        }
        self.emit(Instruction::Construct(args.len() as u16));
        Ok(())
    }
}
