import re

from core.ast import (ASTBinOp, ASTNumber, ASTVariable,
                      ASTPrint, ASTProgramm, ASTAssign,
                      ASTIf, ASTString, ASTNot, ASTBool,
                      ASTInput, ASTType, ASTTypeTrans,
                      ASTRandom, ASTWhile, ASTFor, ASTFunc,
                      ASTFuncCall, ASTReturn, ASTArray,
                      ASTArrayAccess, ASTArraySet, ASTArrayAdd,
                      ASTLen, ASTSop)

import networkx as nx
import matplotlib.pyplot as plt

def print_ast(node, indent=0):
    prefix = '  ' * indent

    if isinstance(node, ASTBinOp):
        print(f'{prefix}BinOp: {node.op}')
        print_ast(node.left, indent + 1)
        print_ast(node.right, indent + 1)
    elif isinstance(node, ASTNot):
        print(f'{prefix}Not:')
        print_ast(node.value, indent + 1)
    elif isinstance(node, ASTType):
        print(f'{prefix}Type: ')
        print_ast(node.value, indent + 1)
    elif isinstance(node, ASTTypeTrans):
        print(f'{prefix}TypeTrans: {node.value}')
    elif isinstance(node, ASTRandom):
        print(f'{prefix}Random:')
    elif isinstance(node, ASTNumber):
        print(f'{prefix}Number: {node.value}')
    elif isinstance(node, ASTString):
        print(f'{prefix}String: {node.value}')
    elif isinstance(node, ASTBool):
        print(f'{prefix}Bool: {node.value}')
    elif isinstance(node, ASTVariable):
        print(f'{prefix}Variable: {node.name}')
    elif isinstance(node, ASTPrint):
        print(f'{prefix}Print:')
        print_ast(node.expr, indent + 1)
    elif isinstance(node, ASTInput):
        print(f'{prefix}Input:')
        print_ast(node.expr, indent + 1)
    elif isinstance(node, ASTProgramm):
        for statement in node.statements:
            print_ast(statement, indent)
    elif isinstance(node, ASTAssign):
        print(f'{prefix}Assign: {node.name}')
        print_ast(node.expr, indent + 1)
    elif isinstance(node, ASTIf):
        print(f'{prefix}If:')
        print_ast(node.condition, indent + 1)
        print_ast(node.true_branch, indent + 1)
        print_ast(node.false_branch, indent + 1)
    elif isinstance(node, ASTFunc):
        print(f'{prefix}Func: {node.name}')
        print(f'{prefix}Arguments: {node.args}')
        print_ast(node.body, indent + 1)
    elif isinstance(node, ASTFuncCall):
        print(f'{prefix}FuncCall: {node.name}')
        for arg in node.args:
            print_ast(arg, indent + 1)
    elif isinstance(node, ASTWhile):
        print(f'{prefix}While:')
        print_ast(node.condition, indent + 1)
        print_ast(node.body, indent + 1)
    elif isinstance(node, ASTSop):
        print(f'{prefix}Sop:')
        print(f'{prefix}Operator: {node.op}')
        print(f'{prefix}var: {node.var}')
    elif isinstance(node, ASTReturn):
        print(f'{prefix}Return:')
        print_ast(node.expr, indent + 1)
    elif isinstance(node, ASTArray):
        print(f'{prefix}Array:')
        for item in node.value:
            print_ast(item, indent + 1)
    elif isinstance(node, ASTArrayAccess):
        print(f'{prefix}ArrayAccess:')
        print_ast(node.array, indent + 1)
        print_ast(node.index, indent + 1)
    elif isinstance(node, ASTArraySet):
        print(f'{prefix}ArraySet:')
        print_ast(node.array, indent + 1)
        print_ast(node.index, indent + 1)
        print_ast(node.value, indent + 1)
    elif isinstance(node, ASTArrayAdd):
        print(f'{prefix}ArrayAdd:')
        print_ast(node.array, indent + 1)
        print_ast(node.value, indent + 1)
    elif isinstance(node, ASTLen):
        print(f'{prefix}Len:')
        print_ast(node.expr, indent + 1)
    elif isinstance(node, ASTFor):
        print(f'{prefix}For:')
        print_ast(node.var, indent + 1)
        print_ast(node.condition, indent + 1)
        print_ast(node.body, indent + 1)

    elif node:
        raise ValueError(f'Unknown node type: {type(node)}')

def add_node(G, node, parent=None):
    """Рекурсивно добавляем узлы в граф"""
    if node is None:
        return

    # Создаем уникальную метку для узла
    if isinstance(node, (ASTNumber, ASTBool, ASTString)):
        label = f"{node.value}"
    elif isinstance(node, ASTVariable):
        label = f"VAR {node.name}"
    elif isinstance(node, ASTBinOp):
        label = f"{node.op}"
    elif isinstance(node, ASTNot):
        label = "NOT"
    elif isinstance(node, ASTIf):
        label = "IF"
    elif isinstance(node, ASTPrint):
        label = "PRINT"
    elif isinstance(node, ASTInput):
        label = "INPUT"
    elif isinstance(node, ASTWhile):
        label = "WHILE"
    elif isinstance(node, ASTFor):
        label = "FOR"
    elif isinstance(node, ASTAssign):
        label = f"ASSIGN {node.name}"
    elif isinstance(node, ASTProgramm):
        label = "PROGRAM"
    else:
        label = str(type(node).__name__)

    # Добавляем узел с меткой
    G.add_node(id(node), label=label)

    # Связываем с родителем если есть
    if parent is not None:
        G.add_edge(id(parent), id(node))

    # Рекурсивно обрабатываем дочерние узлы
    if isinstance(node, ASTProgramm):
        for stmt in node.statements:
            add_node(G, stmt, node)
    elif isinstance(node, ASTBinOp):
        add_node(G, node.left, node)
        add_node(G, node.right, node)
    elif isinstance(node, ASTNot):
        add_node(G, node.value, node)
    elif isinstance(node, ASTIf):
        add_node(G, node.condition, node)
        add_node(G, node.true_branch, node)
        add_node(G, node.false_branch, node)
    elif isinstance(node, ASTPrint):
        add_node(G, node.expr, node)
    elif isinstance(node, ASTInput):
        add_node(G, node.expr, node)
    elif isinstance(node, ASTAssign):
        add_node(G, node.expr, node)
    elif isinstance(node, ASTWhile):
        add_node(G, node.condition, node)
        add_node(G, node.body, node)
    elif isinstance(node, ASTFor):
        add_node(G, node.var, node)
        add_node(G, node.condition, node)
        add_node(G, node.body, node)
    elif isinstance(node, ASTRandom):
        add_node(G, node.expr1, node)
        add_node(G, node.expr2, node)
    elif isinstance(node, ASTArray):
        for item in node.value:
            add_node(G, item, node)
    elif isinstance(node, ASTArrayAccess):
        add_node(G, node.array, node)
        add_node(G, node.index, node)
    elif isinstance(node, ASTArraySet):
        add_node(G, node.array, node)
        add_node(G, node.index, node)
        add_node(G, node.expr, node)
    elif isinstance(node, ASTArrayAdd):
        add_node(G, node.array, node)
        add_node(G, node.expr, node)
    elif isinstance(node, ASTLen):
        add_node(G, node.expr, node)
    elif isinstance(node, ASTFunc):
        add_node(G, node.body, node)
    elif isinstance(node, ASTFuncCall):
        for arg in node.args:
            add_node(G, arg, node)

    return G


def create_graph(ast):
    # Строим граф
    G = nx.DiGraph()
    add_node(G, ast)

    # Рисуем граф
    pos = nx.bfs_layout(G, start=id(ast))
    labels = {n: G.nodes[n]['label'] for n in G.nodes()}

    plt.figure(figsize=(14, 10))
    nx.draw(G, pos, labels=labels, with_labels=True,
            node_size=1500, node_color="lightgray",
            font_size=10, font_weight="bold", arrows=True)
    plt.title("AST Visualization")
    plt.show()

def camel_to_snake(s):
    s1 = re.sub('(.)([A-Z][a-z]+)', r'\1_\2', s)
    return re.sub('([a-z0-9])([A-Z])', r'\1_\2', s1).lower()
