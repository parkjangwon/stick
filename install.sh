#!/usr/bin/env bash
# ============================================================================
# stick - 3대 OS (macOS, Linux, Windows) 지원 설치/삭제 통합 스크립트
# ============================================================================
set -e

# 터미널 컬러 정의 (가독성 향상)
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

UNINSTALL=0
REPO="parkjangwon/stick"

# ── 인자 파싱 ─────────────────────────────────────────────────────────────
for arg in "$@"; do
    if [ "$arg" = "--remove" ] || [ "$arg" = "--uninstall" ]; then
        UNINSTALL=1
    fi
done

# ── 1. 삭제 처리 (Uninstall) ──────────────────────────────────────────────
if [ $UNINSTALL -eq 1 ]; then
    echo -e "${BLUE}==> 🗑️  stick 삭제를 시작합니다...${NC}"
    
    # 1. 실행 중인 실시간 서비스 중지 및 해제
    if command -v stick >/dev/null 2>&1 || command -v stick.exe >/dev/null 2>&1; then
        echo -e "🛑 실행 중인 서비스를 중지합니다..."
        stick stop >/dev/null 2>&1 || stick.exe stop >/dev/null 2>&1 || true
    fi
    
    # 2. 플랫폼별 잔여 서비스 설정 파일 완전 소거 (방어적 설계)
    echo -e "🧹 서비스 잔여 파일을 정리합니다..."
    rm -f "$HOME/Library/LaunchAgents/com.stick.agent.plist"      # macOS LaunchAgent
    rm -f "$HOME/.config/systemd/user/stick.service"             # Linux systemd unit
    
    # 3. 설치 경로별 바이너리 삭제
    echo -e "🗑️  바이너리를 삭제합니다..."
    
    # cargo로 임시 설치한 기록이 있다면 언인스톨 진행
    if command -v cargo >/dev/null 2>&1; then
        cargo uninstall stick >/dev/null 2>&1 || true
    fi
    
    # 다양한 경로에 존재할 수 있는 바이너리 정리
    rm -f "$HOME/.cargo/bin/stick"
    rm -f "$HOME/.cargo/bin/stick.exe"
    rm -f "$HOME/.local/bin/stick"
    rm -f "$HOME/bin/stick"
    rm -f "$HOME/bin/stick.exe"
    
    if [ -f "/usr/local/bin/stick" ]; then
        if [ -w "/usr/local/bin" ]; then
            rm -f "/usr/local/bin/stick"
        else
            echo -e "${YELLOW}관리자 권한(sudo)을 사용해 /usr/local/bin/stick 을 삭제합니다...${NC}"
            sudo rm -f "/usr/local/bin/stick" || true
        fi
    fi
    
    # 4. 설정 및 로그 디렉토리 삭제
    echo -e "🧹 설정 및 로그 디렉토리를 완전히 비웁니다..."
    rm -rf "$HOME/.config/stick"
    rm -rf "$HOME/logs/stick"
    
    echo -e "${GREEN}==> ✅ stick이 시스템에서 성공적으로 삭제되었습니다!${NC}"
    exit 0
fi

# ── 2. 설치 처리 (Install) ────────────────────────────────────────────────
echo -e "${BLUE}==> 🚀 stick 설치를 시작합니다...${NC}"

# OS 및 CPU 아키텍처 자동 판별
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Darwin) 
        OS_LOWER="darwin" 
        ;;
    Linux) 
        OS_LOWER="linux" 
        ;;
    MINGW*|MSYS*|CYGWIN*|Windows_NT) 
        OS_LOWER="windows" 
        ;;
    *)
        echo -e "${RED}⚠️  지원하지 않는 운영체제입니다: $OS${NC}"
        echo -e "직접 소스코드를 빌드하려면 'cargo install --git https://github.com/${REPO}.git' 을 실행해주세요."
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64|amd64) 
        ARCH_LOWER="x86_64" 
        ;;
    arm64|aarch64) 
        ARCH_LOWER="arm64" 
        ;;
    *)
        echo -e "${RED}⚠️  지원하지 않는 CPU 아키텍처입니다: $ARCH${NC}"
        exit 1
        ;;
esac

# 윈도우 환경인 경우 .exe 확장자 적용
if [ "$OS_LOWER" = "windows" ]; then
    BINARY_NAME="stick-${OS_LOWER}-${ARCH_LOWER}.exe"
else
    BINARY_NAME="stick-${OS_LOWER}-${ARCH_LOWER}"
fi

DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${BINARY_NAME}"

echo -e "📦 플랫폼 감지 결과: ${OS_LOWER}-${ARCH_LOWER}"
echo -e "⬇️  최신 릴리즈 바이너리 다운로드 중... (${BINARY_NAME})"

TMP_DIR=$(mktemp -d)
if [ "$OS_LOWER" = "windows" ]; then
    TMP_BIN="${TMP_DIR}/stick.exe"
else
    TMP_BIN="${TMP_DIR}/stick"
fi

# curl로 헤더 수신 성공 및 200 OK 일 때만 파일 저장
if ! curl -sSLf "$DOWNLOAD_URL" -o "$TMP_BIN"; then
    echo -e "${RED}❌ 다운로드 실패!${NC}"
    echo -e "현재해당 플랫폼(${BINARY_NAME})용 최신 릴리즈 바이너리가 등록되지 않았을 수 있습니다."
    echo -e "대안: 소스코드 빌드 방식으로 설치해주세요. (cargo install --git https://github.com/${REPO}.git)"
    rm -rf "$TMP_DIR"
    exit 1
fi

# 실행 권한 부여
chmod +x "$TMP_BIN"

# ── 3. 설치 경로 결정 및 이동 ──────────────────────────────────────────────
if [ "$OS_LOWER" = "windows" ]; then
    INSTALL_DIR="$HOME/bin"
    mkdir -p "$INSTALL_DIR"
    TARGET_BIN="stick.exe"
else
    INSTALL_DIR="/usr/local/bin"
    # 만약 /usr/local/bin 디렉토리가 없거나 일반 유저 권한으로 쓰기가 불가능하다면 로컬 홈경로로 우회
    if [ ! -d "$INSTALL_DIR" ] || [ ! -w "$INSTALL_DIR" ]; then
        INSTALL_DIR="$HOME/.local/bin"
        mkdir -p "$INSTALL_DIR"
    fi
    TARGET_BIN="stick"
fi

echo -e "🚚 바이너리를 안전하게 이송합니다: ${INSTALL_DIR}/${TARGET_BIN}"
if [ "$OS_LOWER" = "windows" ]; then
    mv "$TMP_BIN" "$INSTALL_DIR/$TARGET_BIN"
else
    if [ -w "$INSTALL_DIR" ]; then
        mv "$TMP_BIN" "$INSTALL_DIR/$TARGET_BIN"
    else
        echo -e "${YELLOW}시스템 공용 영역 설치를 위해 관리자 권한(sudo)이 필요합니다:${NC}"
        sudo mv "$TMP_BIN" "$INSTALL_DIR/$TARGET_BIN"
    fi
fi

# 임시 폴더 소거
rm -rf "$TMP_DIR"

# ── 4. 설치 상태 확인 및 완료 안내 ──────────────────────────────────────────
# 윈도우 환경과 유닉스 환경의 명령 작동 체크 유연화
if ! command -v stick >/dev/null 2>&1 && ! command -v stick.exe >/dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  알림: '${INSTALL_DIR}' 디렉토리가 PATH 환경변수에 등록되어 있지 않습니다.${NC}"
    if [ "$OS_LOWER" = "windows" ]; then
        echo -e "윈도우의 '시스템 환경 변수 편집' 메뉴에서 [ ${INSTALL_DIR} ] 경로를 PATH에 추가한 후 다시 실행해주세요."
    else
        echo -e "터미널 셸 설정 파일(예: ~/.zshrc, ~/.bashrc) 끝에 다음 줄을 추가해주세요:"
        echo -e "  ${BLUE}export PATH=\"\$PATH:${INSTALL_DIR}\"${NC}"
    fi
else
    echo -e "${GREEN}==> 🎉 stick 설치가 성공적으로 완수되었습니다!${NC}"
    echo -e ""
    echo -e "아래 명령어를 입력하여 스마트 폴더 설정을 시작하세요:"
    echo -e "  ${YELLOW}stick config${NC}"
    echo -e ""
    echo -e "백그라운드 실시간 수호자 데몬 시작 (macOS / Linux 전용):"
    echo -e "  ${YELLOW}stick start${NC}"
    echo -e ""
    echo -e "언제든지 시스템에서 완전 삭제하려면:"
    echo -e "  ${YELLOW}bash <(curl -sL https://raw.githubusercontent.com/parkjangwon/stick/master/install.sh) --remove${NC}"
fi
