import re

# Update app.rs status messages
with open("src/tui/app.rs", "r") as f:
    app_code = f.read()

replacements = {
    r'"\[↑↓\] 이동  \[Enter\] 선택  \[q\] 저장 후 종료"': r'"↑/↓ 이동  •  Enter 선택  •  Esc 취소"',
    r'"\[Enter\] 확인  \[Esc\] 취소"': r'"Enter 입력 완료  •  Esc 취소"',
    r'"\[↑↓\] 이동  \[Enter\] 폴더 진입  \[Space\] 선택/해제  \[Tab\] 포커스"': r'"↑/↓ 이동  •  Enter 폴더 진입  •  Space 선택/해제  •  Tab 포커스"',
    r'"정말 삭제하시겠습니까\? \[y/n\]"': r'"정말 삭제하시겠습니까? (y/n)"',
    r'"\[↑/↓\] 매칭 순환  \[Enter\] 검색 완료  \[Esc\] 취소"': r'"↑/↓ 매칭 순환  •  Enter 검색 완료  •  Esc 취소"'
}

for old, new in replacements.items():
    app_code = re.sub(old, new, app_code)

with open("src/tui/app.rs", "w") as f:
    f.write(app_code)

print("Updated app.rs")
