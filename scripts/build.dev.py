import subprocess
import os
import shutil
import signal
import sys
import psutil
import mimetypes

mimetypes.add_type("application/javascript", ".js")
mimetypes.add_type("application/wasm", ".wasm")

# Цвета ANSI для терминала
BOLD_WHITE = "\033[1;37m"
RESET = "\033[0m"

PORT = 8080
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
BUILD_DIR = os.path.abspath(os.path.join(SCRIPT_DIR, "..", "build", "dev"))
SRC_LIB = os.path.join(BUILD_DIR, "lib.dev.rs")
SRC_TOML = os.path.join(BUILD_DIR, "Cargo.dev.toml")
DEST_LIB = os.path.abspath(os.path.join(SCRIPT_DIR, "..", "src", "lib.rs"))
DEST_TOML = os.path.abspath(os.path.join(SCRIPT_DIR, "..", "Cargo.toml"))


def info(message):
    print(f"{BOLD_WHITE}[ИНФО]{RESET}: {message}")


def cleanup():
    """Удаление временных файлов"""
    try:
        if os.path.exists(DEST_LIB):
            os.remove(DEST_LIB)
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


def kill_existing_server(port):
    """Завершение процессов, использующих порт"""
    try:
        for proc in psutil.process_iter(["pid", "name"]):
            try:
                for conn in proc.connections(kind="inet"):
                    if conn.status == psutil.CONN_LISTEN and conn.laddr.port == port:
                        info(f"Завершаем процесс {proc.pid}, использующий порт {port}")
                        proc.kill()
                        break
            except (psutil.AccessDenied, psutil.NoSuchProcess):
                continue
    except Exception as e:
        info(f"⚠️ Не удалось завершить процессы на порту {port}: {e}")


def main():
    signal.signal(signal.SIGINT, signal_handler)

    info("📦 Сборка Wasm-пакета...")

    try:
        os.makedirs(os.path.dirname(DEST_LIB), exist_ok=True)
        shutil.copyfile(SRC_LIB, DEST_LIB)
        shutil.copyfile(SRC_TOML, DEST_TOML)

        subprocess.run(["wasm-pack", "build", "--target", "web"], check=True)

        info(f"🔍 Проверяем, свободен ли порт {PORT}...")
        kill_existing_server(PORT)

        info(f"🚀 Запуск локального HTTP-сервера: http://localhost:{PORT}")

        server = subprocess.Popen(
            [
                sys.executable,
                "-m",
                "http.server",
                str(PORT),
                "--bind",
                "127.0.0.1",
                "--directory",
                BUILD_DIR,
            ]
        )

        try:
            server.wait()
        except KeyboardInterrupt:
            info("🛑 Прерывание: Ctrl+C, выключаем сервер...")
            server.terminate()
            server.wait()
            info("✅ Сервер корректно завершён.")
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
