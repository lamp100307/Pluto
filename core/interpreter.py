from random import randint

from core.ast import (ASTBinOp, ASTNumber, ASTVariable,
                      ASTPrint, ASTProgramm, ASTAssign,
                      ASTIf, ASTString, ASTNot, ASTBool,
                      ASTInput, ASTType, ASTTypeTrans,
                      ASTRandom, ASTWhile, ASTFor, ASTFunc,
                      ASTFuncCall, ASTReturn, ASTArray, ASTArrayAccess, ASTArraySet, ASTArrayAdd, ASTLen)

from loguru import logger
from core.utils import camel_to_snake

logger.remove()
logger.add(
    "interpreter.log",
    format='{time:ss:ms} | "{function}" | <level>{level}</level> | {message}',
    level="DEBUG",
    mode='w',
)


class Interpreter:
    def __init__(self):
        self.env = {}
        logger.info("Interpreter initialized")

    def eval(self, node):
        logger.debug(f"Evaluating node: {type(node).__name__}")
        try:
            return self.__getattribute__(f'_eval_{camel_to_snake(type(node).__name__.replace('AST', ''))}')(node)
        except Exception as e:
            logger.error(f"Error evaluating node: {e}")
            raise
        return None

    def _eval_programm(self, node):
        logger.info(f"Evaluating program with {len(node.statements)} statements")
        for i, statement in enumerate(node.statements, 1):
            logger.debug(f"Executing statement {i}/{len(node.statements)}")
            result = self.eval(statement)
            logger.trace(f"Statement {i} result: {result}")
        logger.info("Program execution completed")

    def _eval_assign(self, node):
        value = self.eval(node.expr)
        logger.info(f"Assigning variable '{node.name}' with value: {value}")
        self.env[node.name] = value
        logger.debug(f"Current environment: {self.env}")
        return value

    def _eval_number(self, node):
        logger.trace(f"Number node value: {node.value}")
        return node.value

    def _eval_string(self, node):
        logger.trace(f"String node value: '{node.value}'")
        return node.value

    def _eval_variable(self, node):
        logger.debug(f"Accessing variable '{node.name}'")
        if node.name not in self.env:
            logger.error(f"Undefined variable: '{node.name}'")
            raise NameError(f"Variable '{node.name}' is not defined")
        value = self.env[node.name]
        logger.trace(f"Variable '{node.name}' value: {value}")
        return value

    def _eval_bin_op(self, node):
        logger.debug(f"Binary operation: {node.op}")
        left = self.eval(node.left)
        right = self.eval(node.right)
        logger.trace(f"Operands: {left} {node.op} {right}")

        ops = {
            '+': lambda a, b: a + b,
            '-': lambda a, b: a - b,
            '*': lambda a, b: a * b,
            '/': lambda a, b: a / b,
            '==': lambda a, b: a == b,
            '!=': lambda a, b: a != b,
            '<': lambda a, b: a < b,
            '>': lambda a, b: a > b,
            '<=': lambda a, b: a <= b,
            '>=': lambda a, b: a >= b,
            'and': lambda a, b: a and b,
            'or': lambda a, b: a or b,
            'xor': lambda a, b: a ^ b
        }

        if node.op not in ops:
            logger.error(f"Unknown operator: '{node.op}'")
            raise ValueError(f"Unknown operator: '{node.op}'")

        result = ops[node.op](left, right)
        logger.debug(f"Binary operation result: {left} {node.op} {right} = {result}")
        return result

    def _eval_print(self, node):
        value = self.eval(node.expr)
        logger.info(f"Printing: {value}")
        print(value)
        return value

    def _eval_if(self, node):
        condition = self.eval(node.condition)
        logger.debug(f"If condition: {condition}")

        if condition:
            logger.info("Executing true branch")
            return self.eval(node.true_branch)
        else:
            if node.false_branch:
                logger.info("Executing false branch")
                return self.eval(node.false_branch)
            logger.info("Condition false, no else branch")
        return None

    def _eval_not(self, node):
        value = self.eval(node.value)
        result = not value
        logger.debug(f"Not operation: not {value} = {result}")
        return result

    def _eval_bool(self, node):
        logger.trace(f"Bool node value: {node.value}")
        return node.value

    def _eval_input(self, node):
        logger.info("Waiting for user input")
        if isinstance(node.expr, ASTString):
            input_value = input(node.expr.value)
            logger.debug(f"User input: {input_value}")
            return input_value
        else:
            raise ValueError("Input expression must be a string")

    def _eval_type(self, node):
        logger.trace(f"Type node value: {node.value}")
        value = self.eval(node.value)
        return type(value).__name__

    def _eval_type_trans(self, node):
        logger.trace(f"Type transformation node value: {node.value}")
        value = self.eval(node.value)
        name = node.name
        match name:
            case 'int':
                return int(value)
            case 'str':
                return str(value)
            case 'bool':
                return bool(value)
            case 'array':
                return list(value)
            case _:
                raise ValueError(f"Unknown type transformation: {name}")

    def _eval_random(self, node):
        logger.info("Generating random number")
        min_value = self.eval(node.expr1)
        max_value = self.eval(node.expr2)
        return randint(min_value, max_value)

    def _eval_while(self, node):
        logger.info("Executing while loop")
        while self.eval(node.condition):
            self.eval(node.body)
        logger.info("While loop execution completed")

    def _eval_for(self, node):
        logger.info("Executing for loop")
        var = node.var
        condition = node.condition
        body = node.body

        self.eval(var)

        var_name = node.var.name

        while self.eval(condition):
            self.eval(body)
            current_value = self.env.get(var_name, 0)
            self.env[var_name] = current_value
        logger.info("For loop execution completed")

    def _eval_func(self, node):
        logger.info(f"Defining function: {node.name}")
        func_name = node.name
        args = node.args
        logger.debug(f"Function arguments: {args}")

        self.env[func_name] = (args, node.body)
        logger.debug(f"Function '{func_name}' defined")

    def _eval_func_call(self, node):
        logger.info(f"Calling function: {node.name}")
        func_name = node.name
        args = node.args
        logger.debug(f"Call arguments: {args}")

        func_def = self.env.get(func_name)
        if func_def is None:
            logger.error(f"Function '{func_name}' not found")
            raise NameError(f"Function '{func_name}' not found")

        func_args, func_body = func_def

        if len(args) != len(func_args):
            logger.error(f"Function '{func_name}' expects {len(func_args)} arguments, got {len(args)}")
            raise TypeError(f"Function '{func_name}' expects {len(func_args)} arguments, got {len(args)}")

        old_env = self.env.copy()

        try:
            new_env = {}
            for arg_node, (arg_name, *arg_type) in zip(args, func_args):
                arg_value = self.eval(arg_node)

                if arg_type:
                    expected_type = arg_type[0]
                    actual_type = type(arg_value).__name__
                    if expected_type != actual_type:
                        logger.error(f"Argument '{arg_name}' expects type '{expected_type}', got '{actual_type}'")
                        raise TypeError(f"Argument '{arg_name}' expects type '{expected_type}', got '{actual_type}'")

                new_env[arg_name] = arg_value

            self.env = {**self.env, **new_env}
            logger.debug(f"Function call environment: {self.env}")

            result = None
            for stmt in func_body.statements:
                result = self.eval(stmt)
                if isinstance(stmt, ASTReturn):
                    break

            return result
        finally:
            self.env = old_env

    def _eval_return(self, node):
        logger.info("Evaluating return statement")
        return self.eval(node.expr)

    def _eval_array(self, node):
        logger.trace(f"Array node value: {node.value}")
        return [self.eval(item) for item in node.value]

    def _eval_array_access(self, node):
        array = self.eval(node.array)
        index = self.eval(node.index)

        if not isinstance(array, list) and not isinstance(array, str):
            raise TypeError(f"Expected array, got {type(array).__name__}")
        if not isinstance(index, int):
            raise TypeError(f"Array index must be integer, got {type(index).__name__}")
        if index < 0 or index >= len(array):
            raise IndexError(f"Array index {index} out of bounds")

        return array[index]

    def _eval_array_set(self, node):
        array = self.env.get(node.array)
        index = self.eval(node.index)
        value = self.eval(node.expr)

        if not isinstance(array, list) and not isinstance(array, str):
            raise TypeError(f"Expected array, got {type(array).__name__}")
        if not isinstance(index, int):
            raise TypeError(f"Array index must be integer, got {type(index).__name__}")
        if index < 0 or index >= len(array):
            raise IndexError(f"Array index {index} out of bounds")

        if isinstance(array, str):
            array = array.replace(array[index], value)
        else:
            array[index] = value

        self.env[node.array] = array

    def _eval_array_add(self, node):
        array = self.env.get(node.array)
        value = self.eval(node.expr)

        if not isinstance(array, list):
            raise TypeError(f"Expected array, got {type(array).__name__}")

        array.append(value)

    def _eval_len(self, node):
        return len(self.eval(node.expr))

    def _eval_sop(self, node):
        name = node.var
        val = node.op
        if name in self.env:
            match val:
                case "++":
                    self.env[name] += 1
                case "--":
                    self.env[name] -= 1
        else:
            raise NameError(f"Variable '{name}' is not defined")