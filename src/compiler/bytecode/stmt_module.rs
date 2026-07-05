use super::CodeGenerator;
use crate::compiler::parser::{ExportDeclarationKind, Statement};
use crate::compiler::Instruction;
use crate::errors::Result;

impl CodeGenerator {
    pub(super) fn generate_module_statement(&mut self, stmt: &Statement) -> Result<bool> {
        match stmt {
            Statement::ImportDeclaration { specifiers, source } => {
                let is_native = source.ends_with(".native");
                if is_native {
                    if specifiers.len() == 1 {
                        let local_name = specifiers[0].local.clone();
                        self.emit(Instruction::NativeImport(source.clone(), local_name));
                    } else if specifiers.is_empty() {
                        self.emit(Instruction::NativeImport(
                            source.clone(),
                            "__module".to_string(),
                        ));
                    } else {
                        for spec in specifiers {
                            let local_name = spec.local.clone();
                            self.emit(Instruction::NativeImport(source.clone(), local_name));
                        }
                    }
                } else if specifiers.is_empty() {
                    self.emit(Instruction::ImportModule(source.clone()));
                } else if specifiers.len() == 1 && specifiers[0].imported.as_deref() == Some("*") {
                    self.emit(Instruction::ImportAll(
                        source.clone(),
                        specifiers[0].local.clone(),
                    ));
                } else {
                    for spec in specifiers {
                        let imported_name =
                            spec.imported.clone().unwrap_or_else(|| spec.local.clone());
                        if imported_name == "default" {
                            self.emit(Instruction::ImportDefault(
                                source.clone(),
                                spec.local.clone(),
                            ));
                        } else {
                            self.emit(Instruction::ImportNamed(
                                source.clone(),
                                imported_name,
                                spec.local.clone(),
                            ));
                        }
                    }
                }
                Ok(true)
            }
            Statement::ExportDeclaration { kind } => match kind {
                ExportDeclarationKind::Local(declaration) => {
                    self.record_line_from_span(&declaration.span);
                    match &declaration.inner {
                        Statement::VariableDeclaration { declarations, .. } => {
                            let names: Vec<String> = declarations
                                .iter()
                                .filter_map(|d| Self::extract_identifier_from_pattern(&d.id))
                                .collect();
                            self.generate_statement(&declaration.inner, false)?;
                            for name in &names {
                                self.emit(Instruction::StoreModuleExport(name.clone()));
                            }
                        }
                        Statement::FunctionDeclaration { name, .. } => {
                            self.generate_statement(&declaration.inner, false)?;
                            self.emit(Instruction::StoreModuleExport(name.clone()));
                        }
                        Statement::ClassDeclaration { name, .. } => {
                            self.generate_statement(&declaration.inner, false)?;
                            self.emit(Instruction::StoreModuleExport(name.clone()));
                        }
                        _ => {
                            self.generate_statement(&declaration.inner, false)?;
                        }
                    }
                    Ok(true)
                }
                ExportDeclarationKind::ReExport { specifiers, source } => {
                    if source.is_empty() {
                        for spec in specifiers {
                            let local_name = &spec.local;
                            let exported_name = spec.exported.as_ref().unwrap_or(&spec.local);
                            if exported_name != local_name {
                                self.emit(Instruction::LoadGlobal(local_name.clone()));
                                self.emit(Instruction::StoreGlobal(exported_name.clone()));
                            }
                            self.emit(Instruction::StoreModuleExport(exported_name.clone()));
                        }
                    } else if specifiers.len() == 1
                        && specifiers[0].local == "*"
                        && specifiers[0].exported.as_deref() == Some("*")
                    {
                        self.emit(Instruction::ReExportAll(source.clone()));
                    } else if specifiers.len() == 1 && specifiers[0].local == "*" {
                        let alias = specifiers[0].exported.as_ref().unwrap();
                        self.emit(Instruction::ImportAll(source.clone(), alias.clone()));
                        self.emit(Instruction::StoreModuleExport(alias.clone()));
                    } else {
                        self.emit(Instruction::ImportModule(source.clone()));
                        for spec in specifiers {
                            let imported_name = &spec.local;
                            let exported_name = spec.exported.as_ref().unwrap_or(&spec.local);
                            self.emit(Instruction::ImportNamed(
                                source.clone(),
                                imported_name.clone(),
                                exported_name.clone(),
                            ));
                            self.emit(Instruction::StoreModuleExport(exported_name.clone()));
                        }
                    }
                    Ok(true)
                }
            },
            Statement::InterfaceDeclaration { .. }
            | Statement::TypeAliasDeclaration { .. }
            | Statement::EnumDeclaration { .. } => Ok(true),
            Statement::ExportDefaultDeclaration { declaration } => {
                self.record_line_from_span(&declaration.span);
                match &declaration.inner {
                    Statement::FunctionDeclaration { name, .. } => {
                        self.generate_statement(&declaration.inner, false)?;
                        self.emit(Instruction::StoreModuleExport(name.clone()));
                        self.emit(Instruction::LoadGlobal(name.clone()));
                        self.emit(Instruction::ExportDefault);
                    }
                    Statement::ClassDeclaration { name, .. } => {
                        self.generate_statement(&declaration.inner, false)?;
                        self.emit(Instruction::StoreModuleExport(name.clone()));
                        self.emit(Instruction::LoadGlobal(name.clone()));
                        self.emit(Instruction::ExportDefault);
                    }
                    _ => {
                        self.generate_statement(&declaration.inner, true)?;
                        self.emit(Instruction::ExportDefault);
                    }
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
