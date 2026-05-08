<div align="center">
  <h1>✨ stick ✨</h1>
  <p><strong>macOS/Linux 한글 파일명 NFC 정규화 도구 🇰🇷</strong></p>

  [![Rust](https://img.shields.io/badge/rust-v1.70%2B-orange.svg?logo=rust)](https://www.rust-lang.org/)
  [![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux-lightgrey.svg)]()
  [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

</div>

<br>

## 💡 왜 `stick`이 필요한가요?

**macOS**에서 파일을 생성하면 한글 파일명이 **NFD (자소 분리, 예: `ᄀ`+`ᅡ`)** 형태로 저장됩니다.
이 파일을 그대로 **Windows**나 다른 운영체제로 전송하면 한글 파일명이 보기 흉하게 **깨지는 문제**가 발생합니다.

`stick`은 지정된 폴더를 모니터링하며, 이런 자소 분리된 한글 파일명을 Windows와 호환되는 **NFC (자소 결합, 예: `가`)** 형태로 **자동 변환**해주는 스마트한 백그라운드 도구입니다.

---

## 🚀 주요 기능

- 🔍 **하이브리드 감시**: 실시간 이벤트 감지(`notify`)와 주기적 전체 스캔을 병행하여 파일 변경을 절대 놓치지 않습니다.
- ⚙️ **백그라운드 데몬**: macOS(`launchd`)와 Linux(`systemd`) 서비스 매니저를 자동 감지하여 안정적인 백그라운드 실행을 보장합니다.
- 🖥️ **한글 TUI 설정**: 직관적이고 예쁜 터미널 인터페이스(TUI)로 감시 폴더, 제외 규칙, 스캔 옵션 등을 쉽게 설정할 수 있습니다.
- 🛡️ **안전한 변환**: macOS의 APFS/HFS+ 파일시스템 특성(inode)을 고려하여 안전하게 이름을 변경하며, 숨김 파일이나 임시 파일은 자동으로 건너뜁니다.
- 📋 **상세한 로깅**: 일별 로그 로테이션 기능으로 어떤 파일이 언제 변환되었는지 투명하게 기록합니다.

---

## 🛠️ 설치 방법

간편한 터미널 **원라인 설치**를 지원합니다.

```bash
bash <(curl -sL https://raw.githubusercontent.com/parkjangwon/stick/master/install.sh)
```

> **완전 삭제가 필요하신가요?**
> 바이너리, 데몬 서비스, 설정 및 로그 파일까지 모두 깨끗하게 제거합니다.
> ```bash
> bash <(curl -sL https://raw.githubusercontent.com/parkjangwon/stick/master/install.sh) --remove
> ```

---

## 📖 사용 방법

`stick`은 명령어 기반과 대화형 설정 메뉴를 모두 지원합니다.

### 1. 설정하기 (TUI)
처음 실행 시 반드시 감시할 폴더를 등록해야 합니다.
```bash
stick config
```
> 터미널에서 방향키와 `Enter`를 사용해 직관적으로 감시 폴더를 추가하고, 제외할 확장자나 디렉토리를 설정할 수 있습니다. 설정은 `s` 키를 눌러 저장합니다.

### 2. 수동 스캔 및 변환
원할 때 한 번만 전체 폴더를 검사하고 변환합니다.
```bash
# 실제 변경 없이 어떤 파일이 변환될지 미리보기만 확인
stick scan --dry-run

# 변경 대상을 확인하고 즉시 변환 (확인 프롬프트 건너뛰기)
stick scan --yes
```

### 3. 백그라운드 데몬 시작
설정한 폴더를 실시간으로 감시하며 자동으로 변환하는 서비스를 켭니다.
```bash
# 데몬 서비스 등록 및 시작 (launchd / systemd 자동 감지)
stick start

# 실행 상태 확인
stick status

# 데몬 서비스 중지 및 제거
stick stop
```

---

## ⚙️ 설정 구조

모든 설정 파일은 자동으로 `~/.config/stick/config.json` 경로에 저장되며, 직접 수정할 수도 있습니다.
로그 파일은 기본적으로 `~/logs/stick/` 디렉토리에 일별로 저장됩니다. (`stick_YYYY-MM-DD.log`)

### 기본 제외 대상
- `.`으로 시작하는 숨김 파일 및 디렉토리 (예: `.git`, `.DS_Store`)
- 심볼릭 링크
- `~`로 끝나는 임시 파일
- 기타 설정된 무시 확장자 및 폴더 (`node_modules`, `.tmp` 등)

---

## 📝 구조 및 동작 원리

- `unicode-normalization` 크레이트를 통해 빠르고 정확하게 NFD → NFC 유니코드 정규화를 수행합니다.
- 파일 및 디렉토리 모두 변환 대상에 포함되며, 디렉토리 구조가 깨지지 않도록 반드시 **하위(Leaf)부터 상위(Root) 순서**로 변환합니다.

---

<div align="center">
  <i>Built with ❤️ by <a href="https://github.com/parkjangwon">parkjangwon</a></i>
</div>
