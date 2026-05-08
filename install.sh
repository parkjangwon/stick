#!/usr/bin/env bash
set -e

# 색상 정의
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

UNINSTALL=0

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
    if command -v cargo >/dev/null 2>&1; then
        cargo uninstall stick >/dev/null 2>&1 || true
    fi
    rm -f ~/.cargo/bin/stick
    rm -f /usr/local/bin/stick
    
    # 3. 설정 및 로그 디렉토리 삭제
    echo -e "🧹 설정 및 로그 파일을 삭제합니다..."
    rm -rf ~/.config/stick
    rm -rf ~/logs/stick
    
    echo -e "${GREEN}==> ✅ stick이 시스템에서 완전히 삭제되었습니다.${NC}"
    exit 0
fi

# 설치 처리
echo -e "${BLUE}==> 🚀 stick 설치를 시작합니다...${NC}"

# Rust(cargo) 존재 여부 확인
if ! command -v cargo >/dev/null 2>&1; then
    echo -e "${YELLOW}Rust(cargo)가 설치되어 있지 않습니다.${NC}"
    echo -e "Rust(rustup) 자동 설치를 진행합니다..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    
    # 현재 쉘에 환경변수 적용
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    fi
fi

# 다시 한 번 cargo 확인
if ! command -v cargo >/dev/null 2>&1; then
    echo -e "${RED}Rust 환경 변수를 로드할 수 없습니다. 쉘을 재시작한 후 다시 시도해주세요.${NC}"
    exit 1
fi

echo -e "${BLUE}==> 📦 저장소에서 소스코드를 받아 최적화 빌드를 진행합니다 (1~2분 소요)...${NC}"
cargo install --git https://github.com/parkjangwon/stick.git --force

echo -e "${GREEN}==> 🎉 stick 설치가 성공적으로 완료되었습니다!${NC}"
echo -e ""
echo -e "아래 명령어를 입력하여 설정을 시작하세요:"
echo -e "  ${YELLOW}stick config${NC}"
echo -e ""
echo -e "삭제가 필요한 경우:"
echo -e "  ${YELLOW}bash <(curl -s https://raw.githubusercontent.com/parkjangwon/stick/master/install.sh) --remove${NC}"
