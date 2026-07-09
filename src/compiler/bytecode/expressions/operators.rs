use super::*;

impl CodeGenerator {
    pub(super) fn generate_identifier(&mut self, name: &str) -> Result<()> {
        if name == "this" {
            self.emit(Instruction::LoadThis);
        } else if let Some(local_idx) = self.resolve_local(name) {
            self.emit(Instruction::LoadLocal(local_idx));
        } else {
            self.emit(Instruction::LoadGlobal(name.to_string()));
        }
        Ok(())
    }

    pub(super) fn generate_binary_op_expr(
        &mut self,
        op: &BinaryOperator,
        left: &Expression,
        right: &Expression,
    ) -> Result<()> {
        match op {
            BinaryOperator::And => {
                self.generate_expression(left)?;
                self.emit(Instruction::Dup);
                let skip_right = self.instructions.len();
                self.emit(Instruction::JumpIfNot(0));
                self.emit(Instruction::Pop);
                self.generate_expression(right)?;
                let done = self.instructions.len();
                self.emit(Instruction::Jump(0));
                self.patch_jump(skip_right, self.instructions.len());
                self.patch_jump(done, self.instructions.len());
            }
            BinaryOperator::Or => {
                self.generate_expression(left)?;
                self.emit(Instruction::Dup);
                let skip_right = self.instructions.len();
                self.emit(Instruction::JumpIf(0));
                self.emit(Instruction::Pop);
                self.generate_expression(right)?;
                let done = self.instructions.len();
                self.emit(Instruction::Jump(0));
                self.patch_jump(skip_right, self.instructions.len());
                self.patch_jump(done, self.instructions.len());
            }
            BinaryOperator::NullishCoalescing => {
                self.generate_expression(left)?;
                let skip_right = self.instructions.len();
                self.emit(Instruction::JumpIfNotUndefined(0));
                self.emit(Instruction::Pop);
                self.generate_expression(right)?;
                let done = self.instructions.len();
                self.emit(Instruction::Jump(0));
                self.patch_jump(skip_right, self.instructions.len());
                self.patch_jump(done, self.instructions.len());
            }
            _ => {
                self.generate_expression(left)?;
                self.generate_expression(right)?;
                self.generate_binary_op(op)?;
            }
        }
        Ok(())
    }

    pub(super) fn generate_unary_op(
        &mut self,
        op: &UnaryOperator,
        operand: &Expression,
    ) -> Result<()> {
        match op {
            UnaryOperator::Delete => {
                if let Expression::Member {
                    object,
                    property,
                    computed,
                } = operand
                {
                    self.generate_expression(object)?;
                    if *computed {
                        self.generate_expression(property)?;
                    } else if let Expression::Identifier(name) = property.as_ref() {
                        let idx = self.add_constant(Value::from_string(name.clone()));
                        self.emit(Instruction::LoadConst(idx));
                    } else {
                        self.generate_expression(property)?;
                    }
                    self.emit(Instruction::Delete);
                } else {
                    self.generate_expression(operand)?;
                    self.emit(Instruction::Pop);
                    self.emit(Instruction::LoadTrue);
                }
            }
            UnaryOperator::Void if matches!(operand, Expression::Assignment { .. }) => {
                self.generate_expression(operand)?;
                self.emit(Instruction::Pop);
                self.emit(Instruction::LoadUndefined);
            }
            _ => {
                if let UnaryOperator::Typeof = op {
                    if let Expression::Identifier(name) = operand {
                        // `this` is parsed as Identifier("this") — must use LoadThis,
                        // not TypeOfGlobal("this") which always yields "undefined".
                        if name == "this" {
                            self.emit(Instruction::LoadThis);
                            self.emit(Instruction::TypeOf);
                        } else if let Some(local_idx) = self.resolve_local(name) {
                            self.emit(Instruction::LoadLocal(local_idx));
                            self.emit(Instruction::TypeOf);
                        } else {
                            self.emit(Instruction::TypeOfGlobal(name.clone()));
                        }
                        return Ok(());
                    }
                }
                self.generate_expression(operand)?;
                match op {
                    UnaryOperator::Negate => self.emit(Instruction::Negate),
                    UnaryOperator::Not => self.emit(Instruction::Not),
                    UnaryOperator::Typeof => self.emit(Instruction::TypeOf),
                    UnaryOperator::Void => self.emit(Instruction::Void),
                    UnaryOperator::BitNot => self.emit(Instruction::BitNot),
                    UnaryOperator::Delete => {}
                    UnaryOperator::UnaryPlus => {}
                }
            }
        }
        Ok(())
    }

    pub(super) fn generate_assignment(
        &mut self,
        target: &Expression,
        value: &Expression,
        op: &Option<CompoundAssignmentOp>,
    ) -> Result<()> {
        if let Some(compound_op) = op {
            self.generate_expression(target)?;
            self.generate_expression(value)?;
            match compound_op {
                CompoundAssignmentOp::AddAssign => self.emit(Instruction::Add),
                CompoundAssignmentOp::SubAssign => self.emit(Instruction::Sub),
                CompoundAssignmentOp::MulAssign => self.emit(Instruction::Mul),
                CompoundAssignmentOp::DivAssign => self.emit(Instruction::Div),
                CompoundAssignmentOp::ModAssign => self.emit(Instruction::Mod),
                CompoundAssignmentOp::AndAssign => self.emit(Instruction::And),
                CompoundAssignmentOp::OrAssign => self.emit(Instruction::Or),
                CompoundAssignmentOp::XorAssign => self.emit(Instruction::BitXor),
                CompoundAssignmentOp::BitAndAssign => self.emit(Instruction::BitAnd),
                CompoundAssignmentOp::BitOrAssign => self.emit(Instruction::BitOr),
                CompoundAssignmentOp::ShiftLeftAssign => self.emit(Instruction::ShiftLeft),
                CompoundAssignmentOp::ShiftRightAssign => self.emit(Instruction::ShiftRight),
                CompoundAssignmentOp::UnsignedShiftRightAssign => {
                    self.emit(Instruction::UnsignedShiftRight)
                }
                CompoundAssignmentOp::NullishCoalescingAssign => {
                    self.emit(Instruction::NullishCoalescing)
                }
            }
            if let Expression::Identifier(name) = target {
                self.emit(Instruction::Dup);
                if let Some(local_idx) = self.resolve_local(name) {
                    self.emit(Instruction::StoreLocal(local_idx));
                } else {
                    self.emit(Instruction::StoreGlobal(name.clone()));
                }
            } else if let Expression::Member {
                object,
                property,
                computed,
            } = target
            {
                self.emit(Instruction::Dup);
                self.generate_expression(object)?;
                if *computed {
                    self.generate_expression(property)?;
                } else if let Expression::Identifier(name) = property.as_ref() {
                    let idx = self.add_constant(Value::from_string(name.clone()));
                    self.emit(Instruction::LoadConst(idx));
                } else {
                    self.generate_expression(property)?;
                }
                self.emit(Instruction::Rot3Right);
                self.emit(Instruction::SetProperty);
                self.emit(Instruction::Pop);
            } else {
                return Err(crate::errors::Error::RuntimeError(
                    "Invalid assignment target".into(),
                ));
            }
        } else {
            if let Expression::Member {
                object,
                property,
                computed,
            } = target
            {
                // Assignment expression result is the RHS value (ES):
                //   `var Safer = safer.Buffer = {}` must bind Safer to `{}`,
                //   not to `safer`. SetProperty leaves the receiver, so Dup
                //   the value and Pop the object afterward (same as compound).
                self.generate_expression(value)?;
                self.emit(Instruction::Dup);
                self.generate_expression(object)?;
                if *computed {
                    self.generate_expression(property)?;
                } else if let Expression::Identifier(name) = property.as_ref() {
                    let idx = self.add_constant(Value::from_string(name.clone()));
                    self.emit(Instruction::LoadConst(idx));
                } else {
                    self.generate_expression(property)?;
                }
                self.emit(Instruction::Rot3Right);
                self.emit(Instruction::SetProperty);
                self.emit(Instruction::Pop);
            } else if let Expression::Identifier(name) = target {
                self.generate_expression(value)?;
                self.emit(Instruction::Dup);
                if let Some(local_idx) = self.resolve_local(name) {
                    self.emit(Instruction::StoreLocal(local_idx));
                } else {
                    self.emit(Instruction::StoreGlobal(name.clone()));
                }
            } else if matches!(
                target,
                Expression::ArrayLiteral { .. } | Expression::ObjectLiteral { .. }
            ) {
                self.generate_expression(value)?;
                self.generate_destructuring_assignment_target(target)?;
            } else {
                return Err(crate::errors::Error::RuntimeError(
                    "Invalid assignment target".into(),
                ));
            }
        }
        Ok(())
    }

    pub(super) fn generate_update(
        &mut self,
        op: &UpdateOperator,
        operand: &Expression,
        prefix: bool,
    ) -> Result<()> {
        if let Expression::Identifier(name) = operand {
            if let Some(local_idx) = self.resolve_local(name) {
                let delta = match op {
                    UpdateOperator::Increment => 1i64,
                    UpdateOperator::Decrement => -1i64,
                };
                if prefix {
                    self.emit(Instruction::IncLocal(local_idx, delta));
                    self.emit(Instruction::LoadLocal(local_idx));
                } else {
                    self.emit(Instruction::LoadLocal(local_idx));
                    self.emit(Instruction::IncLocal(local_idx, delta));
                }
            } else if prefix {
                self.generate_expression(operand)?;
                let one = self.add_constant(Value::Float(1.0));
                self.emit(Instruction::LoadConst(one));
                match op {
                    UpdateOperator::Increment => self.emit(Instruction::Add),
                    UpdateOperator::Decrement => self.emit(Instruction::Sub),
                }
                self.emit(Instruction::StoreGlobal(name.clone()));
            } else {
                self.generate_expression(operand)?;
                self.emit(Instruction::LoadGlobal(name.clone()));
                let one = self.add_constant(Value::Float(1.0));
                self.emit(Instruction::LoadConst(one));
                match op {
                    UpdateOperator::Increment => self.emit(Instruction::Add),
                    UpdateOperator::Decrement => self.emit(Instruction::Sub),
                }
                self.emit(Instruction::StoreGlobal(name.clone()));
            }
        } else if let Expression::Member {
            object,
            property,
            computed,
        } = operand
        {
            self.generate_expression(object)?;
            if *computed {
                self.generate_expression(property)?;
            } else if let Expression::Identifier(name) = property.as_ref() {
                let idx = self.add_constant(Value::from_string(name.clone()));
                self.emit(Instruction::LoadConst(idx));
            } else {
                self.generate_expression(property)?;
            }
            self.emit(Instruction::GetProperty);

            if prefix {
                let one = self.add_constant(Value::Float(1.0));
                self.emit(Instruction::LoadConst(one));
                match op {
                    UpdateOperator::Increment => self.emit(Instruction::Add),
                    UpdateOperator::Decrement => self.emit(Instruction::Sub),
                }
                self.emit(Instruction::Dup);
                self.generate_expression(object)?;
                if *computed {
                    self.generate_expression(property)?;
                } else if let Expression::Identifier(name) = property.as_ref() {
                    let idx = self.add_constant(Value::from_string(name.clone()));
                    self.emit(Instruction::LoadConst(idx));
                } else {
                    self.generate_expression(property)?;
                }
                self.emit(Instruction::Rot3Right);
                self.emit(Instruction::SetProperty);
                self.emit(Instruction::Pop);
            } else {
                self.emit(Instruction::Dup);
                let one = self.add_constant(Value::Float(1.0));
                self.emit(Instruction::LoadConst(one));
                match op {
                    UpdateOperator::Increment => self.emit(Instruction::Add),
                    UpdateOperator::Decrement => self.emit(Instruction::Sub),
                }
                self.generate_expression(object)?;
                if *computed {
                    self.generate_expression(property)?;
                } else if let Expression::Identifier(name) = property.as_ref() {
                    let idx = self.add_constant(Value::from_string(name.clone()));
                    self.emit(Instruction::LoadConst(idx));
                } else {
                    self.generate_expression(property)?;
                }
                self.emit(Instruction::Rot3Right);
                self.emit(Instruction::SetProperty);
                self.emit(Instruction::Pop);
            }
        } else {
            self.generate_expression(operand)?;
        }
        Ok(())
    }

    pub(super) fn generate_conditional(
        &mut self,
        test: &Expression,
        consequent: &Expression,
        alternate: &Expression,
    ) -> Result<()> {
        self.generate_expression(test)?;
        let jump_if_not = self.instructions.len();
        self.emit(Instruction::JumpIfNot(0));
        self.generate_expression(consequent)?;
        let jump_to_end = self.instructions.len();
        self.emit(Instruction::Jump(0));
        self.patch_jump(jump_if_not, self.instructions.len());
        self.generate_expression(alternate)?;
        self.patch_jump(jump_to_end, self.instructions.len());
        Ok(())
    }
}
