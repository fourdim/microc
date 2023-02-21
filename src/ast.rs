pub enum BinaryOpKind {
    Add,
    Sub,
}

pub enum SyscallKind {
    Read,
    Write,
}

pub struct ExprAST {
    kind: ExprKind,
}

pub enum ExprKind {
    IntLiteralExprAST {
        value: i32,
    },
    VariableExprAST {
        name: Box<str>,
    },
    BinaryExprAST {
        op: BinaryOpKind,
        lhs: Box<ExprAST>,
        rhs: Box<ExprAST>,
    },
    SyscallExprAST {
        calle: SyscallKind,
        args: Vec<ExprAST>,
    },
}
