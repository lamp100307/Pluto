import re
from collections import namedtuple

Token = namedtuple('Token', ['type', 'value'])

KEYWORDS = {'print', 'if', 'else', 'not',
            'input', 'type', 'random', 'while',
            'for', 'func', 'return', 'add', 'len'}

TOKEN_SPECIFICATION = [
    ('NUMBER', r'\d+(\.\d*)?'),  # числа
    ('STRING', r'\"[^\"]*\"'),
    ('BOOL', r'true|false'),
    ('SOP', r'\+\+|\-\-'),
    ('OP', r'[:]=|==|!=|<=|>=|[+\-*/=<>]|or|and|xor'),  # операции
    ('ID', r'[a-zA-Z_][a-zA-Z_0-9]*'),  # идентификаторы
    ('SKIP', r'[ \t]+'),  # пробелы и табуляция
    ('NEWLINE', r'\n'),  # новая строка
    ('LPAREN', r'\('),  # открывающая скобка
    ('RPAREN', r'\)'),  # закрывающая скобка
    ('LBRACE', r'\{'),
    ('RBRACE', r'\}'),
    ('LBRACKET', r'\['),
    ('RBRACKET', r'\]'),
    ('COMMA', r','),  # запятая
    ('COLON', ':'),
    ('DOT', r'\.'),
    ('MISMATCH', r'.'),  # всё остальное
]

def lex(code):
    tok_regex = '|'.join(f'(?P<{name}>{pattern})' for name, pattern in TOKEN_SPECIFICATION)
    tokens = []
    for match in re.finditer(tok_regex, code):
        kind = match.lastgroup
        value = match.group()
        match kind:
            case 'NUMBER':
                tokens.append(Token(kind, float(value) if '.' in value else int(value)))
            case 'STRING':
                tokens.append(Token(kind, value[1:-1]))
            case 'BOOL':
                tokens.append(Token(kind, True if value == 'true' else False))
            case 'ID':
                if value in ['int', 'str', 'bool', 'array'] and value not in KEYWORDS:
                    tokens.append(Token('TYPETRANS', value))
                elif value in KEYWORDS:
                    tokens.append(Token(value.upper(), value))
                else:
                    tokens.append(Token(kind, value))
            case 'NEWLINE' | 'SKIP':
                continue
            case 'LPAREN' | 'RPAREN' | 'LBRACE' | 'RBRACE' | 'COMMA' | 'COLON' | 'LBRACKET' | 'RBRACKET' | 'OP' | 'DOT' | 'SOP':
                tokens.append(Token(kind, value))
            case 'MISMATCH':
                raise SyntaxError(f'Unexpected character: {value!r}')
    return tokens