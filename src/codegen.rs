use std::{collections::HashMap, env::set_var, ops::Deref, panic};

use inkwell::{
    AddressSpace,
    builder::Builder,
    context::{self, Context},
    module::Module,
    types::BasicTypeEnum,
    values::{BasicValueEnum, FunctionValue, PointerValue},
};

use crate::{
    expr::{self, Expr, LiteralValue},
    stmt::Stmt,
    token::Literal,
    tokentype::TokenType,
};

pub struct Compiler<'ctx> {
    pub context: &'ctx Context,
    pub builder: Builder<'ctx>,
    pub module: Module<'ctx>,

    variables: HashMap<String, (TokenType, BasicTypeEnum<'ctx>, PointerValue<'ctx>)>,

    print_f: FunctionValue<'ctx>,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        let printf_type = context
            .i8_type()
            .fn_type(&[context.ptr_type(AddressSpace::default()).into()], true);
        let print_f = module.add_function("printf", printf_type, None);

        Compiler {
            context,
            builder,
            module,
            variables: HashMap::new(),
            print_f,
        }
    }
    pub fn generate(&mut self, st: Vec<Stmt>) {
        let i32_type = self.context.i32_type();
        let fn_type = i32_type.fn_type(&[], false);
        let function = self.module.add_function("main", fn_type, None);
        let basic_block = self.context.append_basic_block(function, "entry");

        self.builder.position_at_end(basic_block);

        for statement in st {
            self.compile_statement(statement, function);
        }

        // returning 0
        let _ = self
            .builder
            .build_return(Some(&i32_type.const_int(0, false)));
    }

    pub fn compile_statement(&mut self, st: Stmt, func: FunctionValue<'ctx>) {
        match st {
            Stmt::Var {
                name,
                data_type,
                initializer,
            } => {
                let var_type = self.get_basic_type(data_type);
                let alloca = self.builder.build_alloca(var_type, &name.lexeme).unwrap();

                self.variables
                    .insert(name.lexeme.clone(), (data_type, var_type, alloca));

                let expr_value = self.compile_expr(initializer);
                self.builder.build_store(alloca, expr_value.1).unwrap();
            }

            Stmt::IfElse {
                condition,
                then,
                els,
            } => {
                let condition = self.compile_expr(condition);
                if let Some(else_block) = els
                    && TokenType::INT == condition.0
                {
                    let then_basic_block = self.context.append_basic_block(func, "if_block");
                    let else_basic_block = self.context.append_basic_block(func, "else_block");
                    let merge_basic_block = self.context.append_basic_block(func, "merge_block");

                    self.builder
                        .build_conditional_branch(
                            condition.1.into_int_value(),
                            then_basic_block,
                            else_basic_block,
                        )
                        .unwrap();

                    self.builder.position_at_end(then_basic_block);
                    let Stmt::Block { stmts } = *then else {
                        panic!("Expected block");
                    };
                    for i in stmts {
                        self.compile_statement(i, func);
                    }

                    self.builder
                        .build_unconditional_branch(merge_basic_block)
                        .unwrap();
                    self.builder.position_at_end(else_basic_block);

                    let Stmt::Block { stmts } = else_block.deref() else {
                        panic!("Expected block");
                    };
                    for i in stmts {
                        self.compile_statement(i.clone(), func);
                    }
                    self.builder
                        .build_unconditional_branch(merge_basic_block)
                        .unwrap();
                    self.builder.position_at_end(merge_basic_block);
                } else {
                    {
                        let then_basic_block = self.context.append_basic_block(func, "if_block");

                        let merge_basic_block =
                            self.context.append_basic_block(func, "merge_block");

                        self.builder
                            .build_conditional_branch(
                                condition.1.into_int_value(),
                                then_basic_block,
                                merge_basic_block,
                            )
                            .unwrap();

                        self.builder.position_at_end(then_basic_block);
                        let Stmt::Block { stmts } = *then else {
                            panic!("Expected block");
                        };
                        for i in stmts {
                            self.compile_statement(i, func);
                        }

                        self.builder
                            .build_unconditional_branch(merge_basic_block)
                            .unwrap();
                        self.builder.position_at_end(merge_basic_block);
                    }
                }
            }
            Stmt::WHILE { condition, block } => {
                let before_while = self.context.append_basic_block(func, "before_while");
                let then_while = self.context.append_basic_block(func, "then_while");
                let merge_basic_block = self.context.append_basic_block(func, "merge_block_while");

                self.builder.build_unconditional_branch(before_while).unwrap();
                self.builder.position_at_end(before_while);

                let condition = self.compile_expr(condition);

                if condition.0 == TokenType::INT{
                    self.builder.build_conditional_branch(condition.1.into_int_value(),
                        then_while,
                        merge_basic_block).unwrap();
                    self.builder.position_at_end(then_while);
                    let Stmt::Block { stmts } = *block else {
                            panic!("Expected block");
                    };
                    for i in stmts {
                        self.compile_statement(i, func);
                    }

                self.builder.build_unconditional_branch(before_while).unwrap();
                    
                self.builder.position_at_end(merge_basic_block);
                }else {
                    panic!("Error");
                }
            },
            Stmt::Expression { expression }=>{
                self.compile_expr(expression);
            }
            Stmt::Print { expression } => {
                let expr = self.compile_expr(expression);
                self.build_print_call(expr.1, expr.0);
            }
            _ => panic!("uknown values"),
        }
    }

    pub fn build_print_call(&mut self, value: BasicValueEnum<'ctx>, typ: TokenType) {
        let mut value = value;
        let format_str = match value.clone() {
            BasicValueEnum::FloatValue(_) => self.builder.build_global_string_ptr("%f\n", "fmt"),
            BasicValueEnum::PointerValue(_) => {
                self.builder.build_global_string_ptr("%s\n", "fmt_str")
            }
            BasicValueEnum::IntValue(a) => {
                if typ == TokenType::BOOL {
                    let true_str = self
                        .builder
                        .build_global_string_ptr("true", "true_str")
                        .unwrap()
                        .as_pointer_value();
                    let false_str = self
                        .builder
                        .build_global_string_ptr("false", "false_str")
                        .unwrap()
                        .as_pointer_value();

                    value = self
                        .builder
                        .build_select(a, true_str, false_str, "bool_str")
                        .unwrap();

                    self.builder.build_global_string_ptr("%s\n", "fmt_bool")
                } else {
                    self.builder.build_global_string_ptr("%d\n", "fmt")
                }
            }
            _ => panic!("error 5"),
        }
        .unwrap();
        let val = value;

        self.builder
            .build_call(
                self.print_f,
                &[format_str.as_pointer_value().into(), val.into()],
                "printf",
            )
            .unwrap();
    }
    pub fn compile_expr(&mut self, expr: Expr) -> (TokenType, BasicValueEnum<'ctx>) {
        match expr {
            Expr::Assign { name, value }=>{
                if let Some(name) = self.variables.get(&name.lexeme).cloned(){
                    let val = self.builder.build_load(name.1, name.2, "val").unwrap();
                    let expr = self.compile_expr(*value);
                    self.builder.build_store(name.2, expr.1).unwrap();
                    (name.0,val)
                }else {
                    panic!("uknown variable");
                }
            }
            Expr::Literal { value } => self.compile_value(value),
            Expr::Grouping { expression } => self.compile_expr(*expression),
            Expr::Unary { operator, right } => {
                let val = self.compile_expr(*right);
                let ty = val.1.get_type();
                match (operator.token_type, ty) {
                    (TokenType::MINUS, BasicTypeEnum::FloatType(a)) => {
                        let mul = self.context.f64_type().const_float(-1.0);
                        let value = val.1.into_float_value();
                        let value = self
                            .builder
                            .build_float_mul(value, mul.into(), "mul")
                            .unwrap();
                        (TokenType::INT, value.into())
                    }
                    _ => panic!("error 1"),
                }
            }
            Expr::Variable { name } => {
                if let Some(a) = self.variables.get(&name.lexeme) {
                    let loaded = self.builder.build_load(a.1, a.2, "var").unwrap();
                    (a.0, loaded)
                } else {
                    panic!("no such variable");
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.compile_expr(*left);
                let right = self.compile_expr(*right);

                match (left.1.get_type(), operator.token_type, right.1.get_type()) {
                    (BasicTypeEnum::FloatType(a), TokenType::PLUS, BasicTypeEnum::FloatType(b)) => {
                        (
                            right.0,
                            self.builder
                                .build_float_add(
                                    left.1.into_float_value(),
                                    right.1.into_float_value(),
                                    "add_temp",
                                )
                                .unwrap()
                                .into(),
                        )
                    }
                    (
                        BasicTypeEnum::FloatType(a),
                        TokenType::MINUS,
                        BasicTypeEnum::FloatType(b),
                    ) => (
                        right.0,
                        self.builder
                            .build_float_sub(
                                left.1.into_float_value(),
                                right.1.into_float_value(),
                                "sub_temp",
                            )
                            .unwrap()
                            .into(),
                    ),
                    (BasicTypeEnum::FloatType(a), TokenType::STAR, BasicTypeEnum::FloatType(b)) => {
                        (
                            right.0,
                            self.builder
                                .build_float_mul(
                                    left.1.into_float_value(),
                                    right.1.into_float_value(),
                                    "mul_temp",
                                )
                                .unwrap()
                                .into(),
                        )
                    }
                    (
                        BasicTypeEnum::FloatType(a),
                        TokenType::SLASH,
                        BasicTypeEnum::FloatType(b),
                    ) => (
                        right.0,
                        self.builder
                            .build_float_div(
                                left.1.into_float_value(),
                                right.1.into_float_value(),
                                "div_temp",
                            )
                            .unwrap()
                            .into(),
                    ),
                    (
                        BasicTypeEnum::FloatType(a),
                        TokenType::GREATER,
                        BasicTypeEnum::FloatType(b),
                    ) => (
                        right.0,
                        self.builder
                            .build_float_compare(
                                inkwell::FloatPredicate::OGT,
                                left.1.into_float_value(),
                                right.1.into_float_value(),
                                "div_temp",
                            )
                            .unwrap()
                            .into(),
                    ),
                    (BasicTypeEnum::FloatType(a), TokenType::LESS, BasicTypeEnum::FloatType(b)) => {
                        (
                            right.0,
                            self.builder
                                .build_float_compare(
                                    inkwell::FloatPredicate::OLT,
                                    left.1.into_float_value(),
                                    right.1.into_float_value(),
                                    "div_temp",
                                )
                                .unwrap()
                                .into(),
                        )
                    }
                    (
                        BasicTypeEnum::FloatType(a),
                        TokenType::LESS_EQUAL,
                        BasicTypeEnum::FloatType(b),
                    ) => (
                        right.0,
                        self.builder
                            .build_float_compare(
                                inkwell::FloatPredicate::OLE,
                                left.1.into_float_value(),
                                right.1.into_float_value(),
                                "div_temp",
                            )
                            .unwrap()
                            .into(),
                    ),
                    (
                        BasicTypeEnum::FloatType(a),
                        TokenType::GREATER_EQUAL,
                        BasicTypeEnum::FloatType(b),
                    ) => (
                        right.0,
                        self.builder
                            .build_float_compare(
                                inkwell::FloatPredicate::OGE,
                                left.1.into_float_value(),
                                right.1.into_float_value(),
                                "div_temp",
                            )
                            .unwrap()
                            .into(),
                    ),
                    (
                        BasicTypeEnum::FloatType(a),
                        TokenType::EQUAL,
                        BasicTypeEnum::FloatType(b),
                    ) => (
                        right.0,
                        self.builder
                            .build_float_compare(
                                inkwell::FloatPredicate::OEQ,
                                left.1.into_float_value(),
                                right.1.into_float_value(),
                                "div_temp",
                            )
                            .unwrap()
                            .into(),
                    ),
                    (
                        BasicTypeEnum::FloatType(a),
                        TokenType::BANG_EQUAL,
                        BasicTypeEnum::FloatType(b),
                    ) => (
                        right.0,
                        self.builder
                            .build_float_compare(
                                inkwell::FloatPredicate::ONE,
                                left.1.into_float_value(),
                                right.1.into_float_value(),
                                "div_temp",
                            )
                            .unwrap()
                            .into(),
                    ),
                    _ => panic!("error unmatches types"),
                }
            }

            _ => panic!("error unmatches types"),
        }
    }

    pub fn compile_value(&self, value: LiteralValue) -> (TokenType, BasicValueEnum<'ctx>) {
        match value {
            LiteralValue::Number(a) => (
                TokenType::INT,
                self.context.f64_type().const_float(a).into(),
            ),
            LiteralValue::True => (
                TokenType::BOOL,
                self.context.bool_type().const_int(1, false).into(),
            ),
            LiteralValue::False => (
                TokenType::BOOL,
                self.context.bool_type().const_int(0, false).into(),
            ),
            LiteralValue::StringValue(a) => {
                let str_val = self.builder.build_global_string_ptr(&a, "str");
                (
                    TokenType::STRING,
                    str_val.unwrap().as_pointer_value().into(),
                )
            }
            _ => {
                panic!("uknown type");
            }
        }
    }

    fn load_if_pointer(&mut self, value: BasicValueEnum<'ctx>) -> BasicValueEnum<'ctx> {
        match value {
            BasicValueEnum::PointerValue(ptr) => {
                // load the actual value stored in ptr
                self.builder
                    .build_load(ptr.get_type(), ptr, "loadtmp")
                    .unwrap()
            }
            other => other,
        }
    }
    fn get_basic_type(&self, ty: TokenType) -> BasicTypeEnum<'ctx> {
        match ty {
            TokenType::FLOAT => self.context.f64_type().into(),
            TokenType::BOOL => self.context.bool_type().into(),
            TokenType::STRING => self.context.ptr_type(AddressSpace::default()).into(),
            _ => {
                panic!("invvalid type");
            }
        }
    }
}
