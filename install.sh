#!/usr/bin/env bash
set -e

# 색상 정의
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

UNINSTALL=0
REPO="parkjangwon/stick"

# 인자 파싱
for arg in "$@"; do
    if [ "$arg" = "--remove" ] || [ "$arg" = "--uninstall" ]; then
        UNINSTALL=1
    fi
done

# 삭제 처리
if [ $UNINSTALL -eq 1 ]; then
    echo -e "${BLUE}==> 🗑️  stick 삭제를 시작합니다...${NC}"
    
    # 1. 데몬 중지 및 서비스 제거
    if command -v stick >/dev/null 2>&1; then
        echo -e "🛑 실행 중인 서비스를 중지합니다..."
        stick stop >/dev/null 2>&1 || true
    fi
    
    # 2. 바이너리 삭제
    echo -e "🗑️  바이너리를 삭제합니다..."
    
    # cargo로 설치했던 기록이 있다면 지움
    if command -v cargo >/dev/null 2>&1; then
        cargo uninstall stick >/dev/null 2>&1 || true
    fi
    rm -f "$HOME/.cargo/bin/stick"
    
    # /usr/local/bin 또는 ~/.local/bin 확인
    if [ -f "/usr/local/bin/stick" ]; then
        if [ -w "/usr/local/bin" ]; then
            rm -f "/usr/local/bin/stick"
        else
            sudo rm -f "/usr/local/bin/stick" || true
        fi
    fi
    rm -f "$HOME/.local/bin/stick"
    
    # 3. 설정 및 로그 디렉토리 삭제
    echo -e "🧹 설정 및 로그 파일을 삭제합니다..."
    rm -rf "$HOME/.config/stick"
    rm -rf "$HOME/logs/stick"
    
    echo -e "${GREEN}==> ✅ stick이 시스템에서 완전히 삭제되었습니다.${NC}"
    exit 0
fi

# 설치 처리 (사전 빌드된 바이너리 다운로드)
echo -e "${BLUE}==> 🚀 stick 설치를 시작합니다...${NC}"

# OS 및 아키텍처 판별
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Darwin) OS_LOWER="darwin" ;;
    *)
        echo -e "${RED}⚠️  stick은 macOS 전용 스마트 한글 정규화 도구입니다. (현재 OS: $OS)${NC}"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64) ARCH_LOWER="x86_64" ;;
    arm64|aarch64) ARCH_LOWER="arm64" ;;
    *)
        echo -e "${RED}지원하지 않는 아키텍처입니다: $ARCH${NC}"
        exit 1
        ;;
esac

BINARY_NAME="stick-${OS_LOWER}-${ARCH_LOWER}"
DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${BINARY_NAME}"

echo -e "📦 플랫폼 확인 완료: ${OS_LOWER}-${ARCH_LOWER}"
echo -e "⬇️  최신 바이너리를 다운로드합니다..."

TMP_DIR=$(mktemp -d)
TMP_BIN="${TMP_DIR}/stick"

# curl로 다운로드 시도 (HTTP 200 성공시에만 통과)
if ! curl -sSLf "$DOWNLOAD_URL" -o "$TMP_BIN"; then
    echo -e "${RED}❌ 다운로드 실패! 현재 해당 플랫폼(${BINARY_NAME})용 사전 빌드 바이너리가 등록되지 않았을 수 있습니다.${NC}"
    echo -e "대안: 소스코드 빌드 방식을 사용해 주세요. (cargo install --git https://github.com/${REPO}.git)"
    rm -rf "$TMP_DIR"
    exit 1
fi

chmod +x "$TMP_BIN"

# 설치 경로 결정
INSTALL_DIR="/usr/local/bin"
if [ ! -d "$INSTALL_DIR" ]; then
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
fi

echo -e "🚚 바이너리를 ${INSTALL_DIR} 로 이동합니다..."
if [ -w "$INSTALL_DIR" ]; then
    mv "$TMP_BIN" "$INSTALL_DIR/stick"
else
    echo -e "${YELLOW}관리자 권한(sudo)이 필요합니다:${NC}"
    sudo mv "$TMP_BIN" "$INSTALL_DIR/stick"
fi

rm -rf "$TMP_DIR"

# 경로가 PATH에 있는지 확인
if ! command -v stick >/dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  ${INSTALL_DIR} 디렉토리가 PATH 환경변수에 없습니다.${NC}"
    echo -e "터미널 설정(예: ~/.zshrc)에 'export PATH=\"\$PATH:${INSTALL_DIR}\"' 를 추가해주세요."
else
    echo -e "${GREEN}==> 🎉 stick 설치가 성공적으로 완료되었습니다!${NC}"
    echo -e ""
    echo -e "아래 명령어를 입력하여 설정을 시작하세요:"
    echo -e "  ${YELLOW}stick config${NC}"
    echo -e ""
    echo -e "삭제가 필요한 경우:"
    echo -e "  ${YELLOW}bash <(curl -sL https://raw.githubusercontent.com/parkjangwon/stick/master/install.sh) --remove${NC}"
fi
