from core.Lexer import lex
from core.parser import Parser
from core.utils import create_graph, print_ast
from core.interpreter import Interpreter
import sys

def main(code, debug=False, graph=False):
    tokens = list(lex(code))
    print(tokens) if debug else None
    parser = Parser(tokens)
    ast = parser.parse()
    print_ast(ast) if debug else None
    create_graph(ast) if graph else None
    return ast

if __name__ == '__main__':
    debug = False
    if len(sys.argv) > 1:
        file = sys.argv[1]
        if file.endswith('.pluto'):
            pass
        else:
            raise TypeError('File must be a .pluto file')

        if len(sys.argv) > 2:
            debug = True if sys.argv[2] == '--debug' else False
    else:
        raise FileNotFoundError('No file specified')

    with open(file, 'r', encoding='utf-8 sig') as f:
        code = f.read()

    ast = main(code, debug=debug, graph=False)
    interpreter = Interpreter()
    interpreter.eval(ast)
