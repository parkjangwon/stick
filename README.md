<div align="center">
  <h1>✨ stick ✨</h1>
  <p><strong>macOS, Windows & Linux 전천후 한글 파일명 NFC 정규화 도구 🇰🇷</strong></p>

  [![Rust](https://img.shields.io/badge/rust-v1.70%2B-orange.svg?logo=rust)](https://www.rust-lang.org/)
  [![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-blue.svg)]()
  [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

<img width="1014" height="699" alt="macshot-clipboard-B21DB5AE-9DFF-4709-900C-CDDD71BB0F28" src="https://github.com/user-attachments/assets/6902a4b8-e3d2-4e2f-b886-002b90e4539b" />
</div>

<br>

## 💡 왜 `stick`이 필요한가요?

- **macOS 사용자**: 맥에서 생성하는 한글 파일명은 **NFD (자소 분리, 예: `ᄀ`+`ᅡ`)** 형태로 저장됩니다. 이 파일을 그대로 윈도우나 리눅스로 전송하면 한글 파일명이 보기 흉하게 **찢어지고 깨지는 문제**가 발생합니다. `stick`을 **실시간 백그라운드 감시 데몬**으로 실행하면, 파일 생성 시 자동으로 **NFC (자소 결합, 예: `가`)** 형태로 즉각 자동 변환해 줍니다!
- **Windows / Linux 사용자**: 맥 사용자가 보낸 파일이 이미 한글 자모가 분리되어 깨진 채로 도착했나요? 관리자 권한 없이 `stick`을 사용해 클릭 혹은 터치 한 번으로 지정된 폴더 안의 **깨진 한글 파일명들을 완벽하게 일괄 복구(치료)**할 수 있습니다!

---

## 🚀 주요 기능

- 🔍 **하이브리드 감시**: 실시간 디스크 이벤트 감지(`notify`)와 주기적 스캔을 조화롭게 병행하여 단 하나의 파일명 꼬임도 허용하지 않습니다.
- ⚙️ **백그라운드 데몬 (macOS / Linux)**: macOS(`launchd`/`LaunchAgent`) 및 Linux(`systemd` user unit)의 네이티브 서비스 매니저를 통해 부팅 시 백그라운드에서 부드럽게 자동 실행됩니다.
- 🖥️ **한글 TUI 설정**: 직관적이고 고품격인 터미널 UI(`ratatui`)로 감시 폴더, 무시 규칙, 상세 동작 모드를 간편하게 통제합니다. (Windows PowerShell/CMD 에서도 100% 한글 네이티브 구동!)
  - **최근 활동(Recent Activity) 모니터링**: TUI 메인 화면에서 최근 발생한 변환 이력을 한눈에 확인하여 데몬의 동작 상태를 즉각적으로 파악할 수 있습니다.
  - **실시간 폴더 검색 (Search Looping)**: 폴더 탐색기 안에서 `/` 키를 눌러 폴더명을 타이핑하면 매칭된 폴더 사이를 `위(↑) / 아래(↓)` 방향키로 무한 순환(`Looping`) 브라우징합니다.
- 🔔 **macOS 네이티브 배너 알림 (`enable_notifications`)**: 한글 파일명 NFC 정규화가 완료되는 순간 맥 데스크톱 시스템 배너 알림을 즉시 띄워줍니다. (기본값: 사용 안 함)
- ⏱️ **정밀 디바운스 대기시간 조절 (`debounce_delay_seconds`)**: 파일 생성/쓰기가 진행 중인 도중 변환이 겹치는 부작용을 막기 위해 지정된 시간(기본값: `2초`) 동안 대기 후 변환을 집행하는 똑똑한 완충 알고리즘을 사용합니다.
- 🍏 **TUI 통합 서비스 토글 (`auto_start`)**: 일반 설정에서 엔터 한 번으로 맥 시스템 부팅 시 자동 시작 등록/해제를 끄고 켤 수 있습니다. (기본값: 사용함)
- 🛡️ **안전한 구조적 변환 (Safe Rename)**: OS별 파일시스템 특성을 고려하여 파일 경로 정규화 안정성(canonicalize)을 보장하며, 대상 이름의 파일이 이미 존재하는 파일 충돌 상황 시 덮어쓰지 않고 `_conflict` 꼬리표를 달아 원본 데이터 유실을 완벽히 방지합니다. 변환 시 반드시 **하위(Leaf)부터 상위(Root) 순서**로 정밀 변환합니다.

---

## 🛠️ 설치 및 배포 방법

### 🍏 macOS (간편 원라인 설치)
```bash
bash <(curl -sL https://raw.githubusercontent.com/parkjangwon/stick/master/install.sh)
```
> **완전 삭제가 필요하신가요?**
> ```bash
> bash <(curl -sL https://raw.githubusercontent.com/parkjangwon/stick/master/install.sh) --remove
> ```

### 🪟 Windows / 🐧 Linux (직접 빌드 및 설치)
Rust가 설치되어 있다면 한 줄로 즉시 빌드 및 전역 설치가 가능합니다.
```bash
cargo install --git https://github.com/parkjangwon/stick.git
```
*   **Windows 사용자**: 빌드된 `stick.exe` 파일 하나를 편한 폴더에 두거나 환경 변수(Path)에 등록해 두면 어디서나 바로 사용할 수 있습니다. 관리자 권한은 **전혀 필요하지 않으며**, 자신의 일반 다운로드/바탕화면 등의 폴더에서 바로 사용이 가능합니다.
*   **Linux 사용자**: 빌드가 끝난 `stick` 바이너리를 경로 상에 두고 사용할 수 있습니다. `systemd`를 활용한 백그라운드 서비스 시작(`stick start`) 역시 100% 지원됩니다.

---

## 📖 사용 방법

`stick`은 터미널 서브커맨드와 대화형 설정 메뉴(TUI)를 완벽히 지원합니다.

### 1. 설정 및 UI 제어 (TUI)
감시할 폴더를 등록하거나 여러 프리미엄 설정을 제어합니다.
```bash
stick tui
```
> 터미널 창에서 방향키와 `Enter`를 사용해 감시 폴더를 지정합니다. `q` 또는 `Esc`를 입력하면 모든 설정이 안전하게 자동 저장됩니다.

### 2. 수동 스캔 및 즉시 변환
원할 때 수동으로 특정 감시 영역 전체를 일제 검사하고 정규화합니다.
```bash
# 실제 파일명 변경 없이 어떤 파일들이 변환 대상인지 미리보기 확인
stick scan --dry-run

# 대화형 확인 프롬프트를 건너뛰고 즉각 변환 실행
stick scan --yes
```

### 3. 백그라운드 서비스(LaunchAgent) 제어
실시간 파일 감시 데몬을 백그라운드에서 직접 구동하거나 관리합니다.
```bash
# 실시간 감시 데몬 서비스 등록 및 즉시 시작 (launchd)
stick start

# 현재 구동 중인 데몬의 CPU, 메모리 상태 및 실행 정보 확인
stick status

# 데몬 서비스 중지 및 LaunchAgent에서 제거
stick stop
```

### 4. 버전 정보 확인
```bash
# 현재 설치된 stick의 버전 출력 (예: stick v0.2.2)
stick version
```

---

## ⚙️ 설정 구조

모든 설정 정보는 `~/.config/stick/config.json`에 안전하게 보관됩니다.

### 기본 무시(제외) 대상
- `.`으로 시작하는 숨김 파일 및 디렉토리 (예: `.git`, `.DS_Store`)
- 심볼릭 링크
- `~`로 끝나는 다양한 확장자의 임시 작업 파일
- 사용자가 TUI "제외 설정"에서 지정한 예외 디렉토리 및 특정 확장자 패턴

---

## 📋 로깅 안내

실시간 백그라운드 변환 로그는 `~/logs/stick/` 폴더 하위에 일자별로 자동 로테이션되어 보관됩니다. (예: `stick_YYYY-MM-DD.log`) 로그 레벨(`info`, `debug`) 또한 TUI 로그 설정에서 간편히 조절하실 수 있습니다.

---

<div align="center">
  <i>Built with ❤️ by <a href="https://github.com/parkjangwon">parkjangwon</a></i>
</div>
