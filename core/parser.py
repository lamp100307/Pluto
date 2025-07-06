from core.ast import (ASTBinOp, ASTNumber, ASTString,
                      ASTVariable, ASTPrint, ASTProgramm,
                      ASTAssign, ASTIf, ASTNot, ASTNot, ASTBool,
                      ASTInput, ASTType, ASTTypeTrans, ASTRandom, ASTWhile, ASTFor, ASTFunc, ASTFuncCall, ASTReturn,
                      ASTArray, ASTArrayAccess, ASTArraySet, ASTArrayAdd, ASTLen, ASTSop)


class Parser:
    def __init__(self, tokens):
        self.tokens = tokens
        self.pos = 0
        self.func_names = []

    def peek(self):
        return self.tokens[self.pos] if self.pos < len(self.tokens) else None

    def advance(self):
        self.pos += 1

    def expect(self, token_type):
        tok = self.peek()
        if tok and tok.type == token_type:
            self.advance()
            return tok
        raise SyntaxError(f'Expected {token_type}, got {tok.type if tok else "EOF"}')

    def parse(self):
        statements = []
        while self.peek():
            statements.append(self.parse_statement())
        return ASTProgramm(statements)

    def parse_statement(self):
        if not self.peek():
            return None
        if self.peek().type == 'ID':
            return self.parse_assignment()
        elif self.peek().type == 'LBRACKET':
            return self.parse_array()
        elif self.peek().type == 'PRINT':
            return self.parse_print()
        elif self.peek().type == 'FUNC':
            return self.parse_func()
        elif self.peek().type == 'IF':
            return self.parse_if()
        elif self.peek().type == 'WHILE':
            return self.parse_while()
        elif self.peek().type == 'FOR':
            return self.parse_for()
        elif self.peek().type == 'NOT':
            return self.parse_not()
        elif self.peek().type == 'INPUT':
            return self.parse_input()
        elif self.peek().type == 'TYPE':
            return self.type_parse()
        elif self.peek().type == 'TYPETRANS':
            return self.type_trans()
        elif self.peek().type == 'RANDOM':
            return self.parse_random()
        elif self.peek().type == 'RETURN':
            return self.parse_return()
        else:
            return self.expr()

    def parse_assignment(self):
        name = self.expect('ID').value
        if name in self.func_names:
            return self.parse_func_call(name)
        if self.peek() and self.peek().type == 'OP':
            self.expect('OP')
            expr = self.expr()
            return ASTAssign(name, expr)
        if self.peek() and self.peek().type == 'SOP':
            val = self.expect('SOP').value
            return ASTSop(val, name)
        if self.peek() and self.peek().type == 'LBRACKET':
            return self.parse_array_set(name)
        if self.peek() and self.peek().type == 'DOT':
            self.expect('DOT')
            func_name = self.peek().value
            self.advance()
            match func_name:
                case 'add':
                    self.expect('LPAREN')
                    expr = self.expr()
                    self.expect('RPAREN')
                    return ASTArrayAdd(name, expr)
                case _:
                    raise SyntaxError(f'Method {func_name} is not defined')
        else:
            if self.peek() and self.peek().type == 'LPAREN':
                raise SyntaxError(f'Function {name} is not defined')
            else:
                raise SyntaxError(f'Skipped = in initialization {name}')

    def parse_print(self):
        self.expect('PRINT')
        self.expect('LPAREN')
        expr = self.expr()
        self.expect('RPAREN')
        return ASTPrint(expr)

    def expr(self):
        left = self._comparison_tail(self._term_tail(self.term()))

        if self.peek() and self.peek().type == 'LPAREN':
            return self.parse_func_call(left.name)

        return left

    def _comparison_tail(self, left):
        tok = self.peek()
        if tok and tok.type == 'OP' and tok.value in ('==', '!=', '<', '>', '<=', '>=', 'and', 'or', 'xor'):
            self.advance()
            right = self._term_tail(self.term())
            return ASTBinOp(left, tok.value, right)
        if tok and tok.type == 'SOP' and tok.value in ('++', '--'):
            self.advance()
        return left

    def _term_tail(self, left):
        tok = self.peek()
        if tok and tok.type == 'OP' and tok.value in ('+', '-'):
            self.advance()
            right = self.term()
            return self._term_tail(ASTBinOp(left, tok.value, right))
        return left

    def term(self):
        return self._factor_tail(self.factor())

    def _factor_tail(self, left):
        tok = self.peek()
        if tok and tok.type == 'OP' and tok.value in ('*', '/'):
            self.advance()
            right = self.factor()
            return self._factor_tail(ASTBinOp(left, tok.value, right))
        return left

    def factor(self):
        tok = self.peek()
        if not tok:
            raise SyntaxError("Unexpected end of input")
        if tok.type == 'NUMBER':
            self.advance()
            return ASTNumber(tok.value)
        elif tok.type == 'STRING':
            self.advance()
            return ASTString(tok.value)
        elif tok.type == 'BOOL':
            self.advance()
            return ASTBool(tok.value)
        elif self.peek().type == 'LBRACKET':
            return self.parse_array()
        elif tok.type == 'ID':
            self.advance()
            var = ASTVariable(tok.value)
            if self.peek() and self.peek().type == 'LBRACKET':
                return self.parse_array_access(var)
            return var
        elif tok.type == 'NOT':
            self.advance()
            expr = self.expr()
            return ASTNot(expr)
        elif tok.type == 'INPUT':
            return self.parse_input()
        elif tok.type == 'LPAREN':
            self.advance()
            expr = self.expr()
            self.expect('RPAREN')
            return expr
        elif tok.type == 'TYPE':
            return self.type_parse()
        elif tok.type == 'TYPETRANS':
            return self.type_trans()
        elif self.peek().type == 'RANDOM':
            return self.parse_random()
        elif tok.type == 'LEN':
            return self.parse_len()
        else:
            raise SyntaxError(f'Unexpected token {tok}, pos: {self.pos}')

    def parse_if(self):
        self.expect('IF')
        self.expect('LPAREN')
        condition = self.expr()
        self.expect('RPAREN')
        self.expect('LBRACE')

        # Parse then block
        then_statements = []
        while self.peek() and self.peek().type != 'RBRACE':
            then_statements.append(self.parse_statement())
        self.expect('RBRACE')

        # Parse else block if present
        else_statements = None
        if self.peek() and self.peek().type == 'ELSE':
            self.expect('ELSE')
            self.expect('LBRACE')
            else_statements = []
            while self.peek() and self.peek().type != 'RBRACE':
                else_statements.append(self.parse_statement())
            self.expect('RBRACE')

        then_block = ASTProgramm(then_statements)
        else_block = ASTProgramm(else_statements) if else_statements else None

        return ASTIf(condition, then_block, else_block)

    def parse_not(self):
        self.expect('NOT')
        expr = self.expr()
        return ASTNot(expr)

    def parse_input(self):
        self.expect('INPUT')
        self.expect('LPAREN')
        expr = self.expr()
        self.expect('RPAREN')
        return ASTInput(expr)

    def type_parse(self):
        self.expect('TYPE')
        self.expect('LPAREN')
        expr = self.expr()
        self.expect('RPAREN')
        return ASTType(expr)

    def type_trans(self):
        name = self.expect('TYPETRANS').value
        self.expect('LPAREN')
        expr = self.expr()
        self.expect('RPAREN')
        return ASTTypeTrans(name, expr)

    def parse_random(self):
        self.expect('RANDOM')
        self.expect('LPAREN')
        expr = self.expr()
        self.expect('COMMA')
        expr2 = self.expr()
        self.expect('RPAREN')
        return ASTRandom(expr, expr2)

    def parse_while(self):
        self.expect('WHILE')
        self.expect('LPAREN')
        condition = self.expr()
        self.expect('RPAREN')
        self.expect('LBRACE')
        statements = []
        while self.peek() and self.peek().type != 'RBRACE':
            statements.append(self.parse_statement())
        self.expect('RBRACE')
        return ASTWhile(condition, ASTProgramm(statements))

    def parse_for(self):
        self.expect('FOR')
        self.expect('LPAREN')
        var = self.parse_assignment()
        self.expect('COMMA')
        condition = self.expr()
        self.expect('RPAREN')
        self.expect('LBRACE')
        statements = []
        while self.peek() and self.peek().type != 'RBRACE':
            statements.append(self.parse_statement())
        self.expect('RBRACE')
        return ASTFor(var, condition, ASTProgramm(statements))

    def parse_func(self):
        self.expect('FUNC')
        name = self.expect('ID').value
        self.expect('LPAREN')
        args = []
        while self.peek() and self.peek().type != 'RPAREN':
            var = [self.expect('ID').value]
            if self.peek() and self.peek().type == 'COLON':
                self.expect('COLON')
                var.append(self.expect('TYPETRANS').value)

            if self.peek() and self.peek().type == 'COMMA':
                self.advance()

            args.append(var if len(var) == 2 else var[0])
        self.expect('RPAREN')
        self.expect('LBRACE')
        statements = []
        while self.peek() and self.peek().type != 'RBRACE':
            statements.append(self.parse_statement())
        self.expect('RBRACE')
        self.func_names.append(name)
        return ASTFunc(name, args, ASTProgramm(statements))

    def parse_func_call(self, func_name):
        self.expect('LPAREN')
        args = []
        while self.peek() and self.peek().type != 'RPAREN':
            args.append(self.expr())
            if self.peek() and self.peek().type == 'COMMA':
                self.advance()
        self.expect('RPAREN')
        return ASTFuncCall(func_name, args)

    def parse_return(self):
        self.expect('RETURN')
        self.expect('LPAREN')
        expr = self.expr()
        self.expect('RPAREN')
        return ASTReturn(expr)

    def parse_array(self):
        self.expect('LBRACKET')
        elements = []
        while self.peek() and self.peek().type != 'RBRACKET':
            elements.append(self.expr())
            if self.peek() and self.peek().type == 'COMMA':
                self.advance()
        self.expect('RBRACKET')
        return ASTArray(elements)

    def parse_array_access(self, var):
        self.expect('LBRACKET')
        index = self.expr()
        self.expect('RBRACKET')
        return ASTArrayAccess(var, index)

    def parse_array_set(self, name):
        self.expect('LBRACKET')
        index = self.expr()
        self.expect('RBRACKET')
        if self.expect('OP').value == '=':
            expr = self.expr()
            return ASTArraySet(name, index, expr)
        else:
            raise SyntaxError(f'Skipped = in setting {name}[{index}]')

    def parse_len(self):
        self.expect('LEN')
        self.expect('LPAREN')
        expr = self.expr()
        self.expect('RPAREN')
        return ASTLen(expr)
