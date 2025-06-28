import subprocess
import os
import signal
import sys

# ANSI escape codes
BOLD_WHITE = "\033[1;37m"
RESET = "\033[0m"

PORT = 8080
BUILD_DIR = os.path.join(os.path.dirname(__file__), "..", "build/dev")


def info(message):
    print(f"{BOLD_WHITE}[INFO]{RESET}: {message}")


def cleanup():
    """Clean up copied files"""
    try:
        if os.path.exists("src/lib.rs"):
            subprocess.run("rm src/lib.rs", shell=True, check=True)
        if os.path.exists("Cargo.toml"):
            subprocess.run("rm Cargo.toml", shell=True, check=True)
        info("🧹 Cleaned up temporary files")
    except Exception as e:
        info(f"⚠️  Warning: Could not clean up files: {e}")


def signal_handler(sig, frame):
    """Handle SIGINT (Ctrl+C) signal."""
    info("🛑 Build interrupted by user (Ctrl+C).")
    cleanup()
    sys.exit(1)


def kill_existing_server(port):
    try:
        result = subprocess.run(
            f"lsof -t -i:{port}", shell=True, capture_output=True, text=True
        )
        pids = result.stdout.strip().split()
        for pid in pids:
            info(f"Killing existing process on port {port} (PID {pid})")
            os.kill(int(pid), signal.SIGKILL)
    except Exception as e:
        info(f"Could not kill process on port {port}: {e}")


def main():
    # Register the SIGINT handler
    signal.signal(signal.SIGINT, signal_handler)

    info(f"📦 Building Wasm package...")
    
    try:
        # Copy the source files
        subprocess.run(f"cp {BUILD_DIR}/lib.dev.rs src/lib.rs", shell=True, check=True)
        subprocess.run(f"cp {BUILD_DIR}/Cargo.dev.toml Cargo.toml", shell=True, check=True)

        # Run the build process
        subprocess.run("wasm-pack build --target web", shell=True, check=True)

        info(f"🔍 Ensuring port {PORT} is free...")
        kill_existing_server(PORT)

        info(f"🚀 Starting HTTP server on http://localhost:{PORT}")

        server = subprocess.Popen(
            f"python3 -m http.server {PORT} --bind 127.0.0.1 --directory {BUILD_DIR}",
            shell=True,
        )

        try:
            server.wait()
        except KeyboardInterrupt:
            info(f"🛑 Caught Ctrl+C, shutting down server...")
            server.terminate()
            server.wait()
            info(f"✅ Server shut down cleanly.")

    except subprocess.CalledProcessError as e:
        info(f"❌ Build failed with return code {e.returncode}")
        cleanup()
        sys.exit(e.returncode)
    except Exception as e:
        info(f"❌ Unexpected error: {e}")
        cleanup()
        sys.exit(1)
    finally:
        # Always clean up, even on successful completion
        cleanup()


if __name__ == "__main__":
    main()
