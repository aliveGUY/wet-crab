import subprocess
import os
import signal

# ANSI escape codes
BOLD_WHITE = "\033[1;37m"
RESET = "\033[0m"

PORT = 8080
BUILD_DIR = os.path.join(os.path.dirname(__file__), '..', 'build')

def kill_existing_server(port):
    try:
        result = subprocess.run(
            f"lsof -t -i:{port}",
            shell=True,
            capture_output=True,
            text=True
        )
        pids = result.stdout.strip().split()
        for pid in pids:
            print(f"{BOLD_WHITE}[INFO]{RESET} Killing existing process on port {port} (PID {pid})")
            os.kill(int(pid), signal.SIGKILL)
    except Exception as e:
        print(f"{BOLD_WHITE}[WARN]{RESET} Could not kill process on port {port}: {e}")

def main():
    print(f"{BOLD_WHITE}[INFO]{RESET} 📦 Building Wasm package...")
    subprocess.run(
        'wasm-pack build --target web',
        shell=True,
        check=True
    )

    print(f"{BOLD_WHITE}[INFO]{RESET} 🔍 Ensuring port {PORT} is free...")
    kill_existing_server(PORT)

    print(f"{BOLD_WHITE}[INFO]{RESET} 🚀 Starting HTTP server on http://localhost:{PORT}")
    server = subprocess.Popen(
        f'python3 -m http.server {PORT} --bind 127.0.0.1 --directory {BUILD_DIR}',
        shell=True
    )

    try:
        server.wait()
    except KeyboardInterrupt:
        print(f"\n{BOLD_WHITE}[INFO]{RESET} 🛑 Caught Ctrl+C, shutting down server...")
        server.terminate()
        server.wait()
        print(f"{BOLD_WHITE}[INFO]{RESET} ✅ Server shut down cleanly.")

if __name__ == "__main__":
    main()
