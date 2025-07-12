import subprocess
import os
import signal
import sys

# ANSI escape codes (будут работать в Windows 10+ и в большинстве терминалов)
BOLD_WHITE = "\033[1;37m"
RESET = "\033[0m"

# Пути к исходным и целевым файлам
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
BUILD_DIR = os.path.abspath(os.path.join(SCRIPT_DIR, "..", "build", "windows"))
SRC_MAIN = os.path.join(BUILD_DIR, "main.windows.rs")
SRC_TOML = os.path.join(BUILD_DIR, "Cargo.windows.toml")
DEST_MAIN = os.path.join(SCRIPT_DIR, "..", "src", "main.rs")
DEST_TOML = os.path.join(SCRIPT_DIR, "..", "Cargo.toml")


def info(message):
    print(f"{BOLD_WHITE}[INFO]{RESET}: {message}")


def cleanup():
    """Удаление временных файлов"""
    try:
        if os.path.exists(DEST_MAIN):
            os.remove(DEST_MAIN)
        if os.path.exists(DEST_TOML):
            os.remove(DEST_TOML)
        info("🧹 Временные файлы удалены")
    except Exception as e:
        info(f"⚠️ Не удалось удалить файлы: {e}")


def signal_handler(sig, frame):
    """Обработка Ctrl+C"""
    info("🛑 Сборка прервана пользователем (Ctrl+C).")
    cleanup()
    sys.exit(1)


def main():
    signal.signal(signal.SIGINT, signal_handler)

    info("📦 Сборка под Windows...")

    try:
        os.makedirs(os.path.dirname(DEST_MAIN), exist_ok=True)

        # Копируем нужные файлы
        info("📁 Копирование исходников...")
        subprocess.run(["copy", SRC_MAIN, DEST_MAIN], shell=True, check=True)
        subprocess.run(["copy", SRC_TOML, DEST_TOML], shell=True, check=True)

        # Компиляция
        info("🔨 Компиляция с помощью cargo...")
        subprocess.run("cargo run", shell=True, check=True)

    except subprocess.CalledProcessError as e:
        info(f"❌ Сборка завершилась с ошибкой (код {e.returncode})")
        cleanup()
        sys.exit(e.returncode)
    except Exception as e:
        info(f"❌ Неожиданная ошибка: {e}")
        cleanup()
        sys.exit(1)
    finally:
        cleanup()


if __name__ == "__main__":
    main()
