class ASTNumber:
    def __init__(self, value):
        self.value = value

class ASTBool:
    def __init__(self, value):
        self.value = value

class ASTNot:
    def __init__(self, value):
        self.value = value

class ASTString:
    def __init__(self, value):
        self.value = value

class ASTArray:
    def __init__(self, value):
        self.value = value

class ASTArrayAccess:
    def __init__(self, array, index):
        self.array = array
        self.index = index

class ASTArraySet:
    def __init__(self, array, index, expr):
        self.array = array
        self.index = index
        self.expr = expr

class ASTArrayAdd:
    def __init__(self, array, expr):
        self.array = array
        self.expr = expr

class ASTRandom:
    def __init__(self, expr1, expr2):
        self.expr1 = expr1
        self.expr2 = expr2

class ASTLen:
    def __init__(self, expr):
        self.expr = expr

class ASTType:
    def __init__(self, value):
        self.value = value

class ASTTypeTrans:
    def __init__(self, name, value):
        self.name = name
        self.value = value

class ASTVariable:
    def __init__(self, name):
        self.name = name

class ASTFuncCall:
    def __init__(self, name, args):
        self.name = name
        self.args = args

class ASTReturn:
    def __init__(self, expr):
        self.expr = expr

class ASTFunc:
    def __init__(self, name, args, body):
        self.name = name
        self.args = args
        self.body = body

class ASTIf:
    def __init__(self, condition, true_branch, false_branch):
        self.condition = condition
        self.true_branch = true_branch
        self.false_branch = false_branch

class ASTWhile:
    def __init__(self, condition, body):
        self.condition = condition
        self.body = body

class ASTFor:
    def __init__(self, var, condition, body):
        self.var = var
        self.condition = condition
        self.body = body

class ASTBinOp:
    def __init__(self, left, op, right):
        self.left = left
        self.op = op
        self.right = right

class ASTSop:
    def __init__(self, op, var):
        self.op = op
        self.var = var


class ASTPrint:
    def __init__(self, expr):
        self.expr = expr

class ASTInput:
    def __init__(self, expr):
        self.expr = expr

class ASTAssign:
    def __init__(self, name, expr):
        self.name = name
        self.expr = expr

class ASTProgramm:
    def __init__(self, statements):
        self.statements = statements

