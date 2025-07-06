import os

# Папки и файлы, которые нужно игнорировать
IGNORED_DIRS = {'.git', '.venv', '.idea', '__pycache__', 'node_modules', 'venv', 'env'}
IGNORED_FILE_EXTENSIONS = {'.pyc', '.pyo', '.pyd', '.py~', '.swp', '.tmp', '.bak'}


def should_ignore(path):
    """Проверяет, нужно ли игнорировать файл или папку"""
    # Проверка папок из черного списка
    for ignored_dir in IGNORED_DIRS:
        if ignored_dir in path.split(os.sep):
            return True

    # Проверка расширений файлов
    if os.path.isfile(path):
        _, ext = os.path.splitext(path)
        if ext.lower() in IGNORED_FILE_EXTENSIONS:
            return True

    return False


def print_file_contents(file_path):
    try:
        with open(file_path, 'r', encoding='utf-8') as file:
            content = file.read()
            print(f"\nполный путь до файла: {file_path}")
            print("содержимое файла:")
            print(content)
    except UnicodeDecodeError:
        try:
            with open(file_path, 'r', encoding='latin-1') as file:
                content = file.read()
                print(f"\nполный путь до файла: {file_path}")
                print("содержимое файла (кодировка latin-1):")
                print(content)
        except Exception as e:
            print(f"\nполный путь до файла: {file_path}")
            print(f"не удалось прочитать файл (бинарный или неожиданная кодировка): {e}")
    except Exception as e:
        print(f"\nполный путь до файла: {file_path}")
        print(f"ошибка при чтении файла: {e}")


def process_directory(directory_path):
    for root, dirs, files in os.walk(directory_path, topdown=True):
        # Удаляем игнорируемые папки из списка для обработки
        dirs[:] = [d for d in dirs if not should_ignore(os.path.join(root, d))]

        for file in files:
            file_path = os.path.join(root, file)
            if not should_ignore(file_path):
                print_file_contents(file_path)


if __name__ == "__main__":
    folder_path = input("Введите путь к папке: ").strip()
    if os.path.isdir(folder_path):
        print(f"Чтение содержимого папки: {folder_path}")
        print(f"Игнорируются папки: {', '.join(IGNORED_DIRS)}")
        print(f"Игнорируются файлы с расширениями: {', '.join(IGNORED_FILE_EXTENSIONS)}")
        print("=" * 50)
        process_directory(folder_path)
    else:
        print("Указанный путь не является папкой или не существует.")