// ============================================================================
// stick - TUI 상태 관리 모듈
// 메뉴 구조, 상태 전환, 사용자 입력 처리
// ============================================================================

use crate::config::StickConfig;

/// 현재 표시 중인 화면
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Main,              // 메인 메뉴
    WatchPaths,        // 감시 폴더 관리
    ExcludeSettings,   // 제외 설정
    ExcludeExtensions, // 제외 확장자 관리
    ExcludeDirs,       // 제외 폴더 관리
    LogSettings,       // 로그 설정
    GeneralSettings,   // 일반 설정
    DirPicker,         // 감시 폴더 탐색기
}

#[derive(Debug, Clone, PartialEq)]
pub enum DirPickerFocus {
    List,
    Confirm,
    Cancel,
}

pub struct DirPickerState {
    pub current_dir: std::path::PathBuf,
    pub items: Vec<std::path::PathBuf>,
    pub selected_index: usize,
    pub focus: DirPickerFocus,
    pub selected_paths: std::collections::HashSet<std::path::PathBuf>,
    pub list_state: ratatui::widgets::ListState,
}

impl DirPickerState {
    pub fn new() -> Self {
        let current_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/"));
        let mut state = Self {
            current_dir,
            items: Vec::new(),
            selected_index: 0,
            focus: DirPickerFocus::List,
            selected_paths: std::collections::HashSet::new(),
            list_state: ratatui::widgets::ListState::default(),
        };
        state.refresh_items();
        state
    }

    pub fn refresh_items(&mut self) {
        self.items.clear();
        self.selected_index = 0;
        
        // 부모 디렉토리 가기
        if let Some(parent) = self.current_dir.parent() {
            self.items.push(parent.to_path_buf());
        }
        
        if let Ok(entries) = std::fs::read_dir(&self.current_dir) {
            let mut dirs: Vec<_> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .collect();
            
            dirs.sort_by(|a, b| {
                let a_name = a.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
                let b_name = b.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
                a_name.cmp(&b_name)
            });
            
            self.items.extend(dirs);
        }
        self.list_state.select(Some(0));
    }
}

/// 텍스트 입력의 목적
#[derive(Debug, Clone, PartialEq)]
pub enum InputTarget {
    AddExcludeExtension,
    AddExcludeDir,
    EditLogPath,
    EditScanInterval,
    DirPickerSearch,
}

/// TUI 앱 전체 상태
pub struct App {
    /// 현재 설정 (수정 중)
    pub config: StickConfig,
    /// 현재 화면
    pub current_screen: Screen,
    /// 현재 선택된 메뉴 인덱스
    pub selected_index: usize,
    /// 텍스트 입력 모드 여부
    pub input_mode: bool,
    /// 입력 버퍼
    pub input_buffer: String,
    /// 입력 대상
    pub input_target: Option<InputTarget>,
    /// 저장 여부
    pub should_save: bool,
    /// 하단 상태 메시지
    pub status_message: String,
    /// 삭제 확인 모드
    pub confirm_delete: bool,
    /// 삭제 대상 인덱스
    pub delete_target_index: Option<usize>,
    /// 디렉토리 탐색기 상태
    pub dir_picker: Option<DirPickerState>,
}

impl App {
    pub fn new(config: StickConfig) -> Self {
        Self {
            config,
            current_screen: Screen::Main,
            selected_index: 0,
            input_mode: false,
            input_buffer: String::new(),
            input_target: None,
            should_save: true, // 이제 기본적으로 항상 저장하고 나갑니다.
            status_message: "[↑↓] 이동  [Enter] 선택  [q] 저장 후 종료".to_string(),
            confirm_delete: false,
            delete_target_index: None,
            dir_picker: None,
        }
    }

    /// 현재 화면의 메뉴 항목 수
    pub fn menu_len(&self) -> usize {
        match self.current_screen {
            Screen::Main => 4, // 저장/취소 버튼 제거로 4개로 축소
            Screen::WatchPaths => self.config.watch_paths.len(),
            Screen::ExcludeSettings => 6,
            Screen::ExcludeExtensions => self.config.exclude_extensions.len(),
            Screen::ExcludeDirs => self.config.exclude_directories.len(),
            Screen::LogSettings => 3, // 경로, 레벨, 크기 확인
            Screen::GeneralSettings => 3,
            Screen::DirPicker => 0, // DirPicker는 별도 관리
        }
    }

    /// 커서 위로 이동
    pub fn move_up(&mut self) {
        if self.current_screen == Screen::DirPicker {
            if let Some(dp) = &mut self.dir_picker {
                if dp.focus == DirPickerFocus::List && dp.selected_index > 0 {
                    dp.selected_index -= 1;
                    dp.list_state.select(Some(dp.selected_index));
                }
            }
            return;
        }

        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// 커서 아래로 이동
    pub fn move_down(&mut self) {
        if self.current_screen == Screen::DirPicker {
            if let Some(dp) = &mut self.dir_picker {
                if dp.focus == DirPickerFocus::List && dp.selected_index < dp.items.len().saturating_sub(1) {
                    dp.selected_index += 1;
                    dp.list_state.select(Some(dp.selected_index));
                }
            }
            return;
        }

        let max = self.menu_len();
        if max > 0 && self.selected_index < max - 1 {
            self.selected_index += 1;
        }
    }

    /// 메뉴 항목 선택 (Enter)
    pub fn select(&mut self) {
        match self.current_screen {
            Screen::Main => self.select_main_menu(),
            Screen::ExcludeSettings => self.select_exclude_menu(),
            Screen::LogSettings => self.select_log_menu(),
            Screen::GeneralSettings => self.select_general_menu(),
            _ => {}
        }
    }

    /// 메인 메뉴 선택 처리
    fn select_main_menu(&mut self) {
        match self.selected_index {
            0 => {
                self.current_screen = Screen::WatchPaths;
                self.selected_index = 0;
            }
            1 => {
                self.current_screen = Screen::ExcludeSettings;
                self.selected_index = 0;
            }
            2 => {
                self.current_screen = Screen::LogSettings;
                self.selected_index = 0;
            }
            3 => {
                self.current_screen = Screen::GeneralSettings;
                self.selected_index = 0;
            }
            _ => {}
        }
    }

    /// 제외 설정 메뉴 선택
    fn select_exclude_menu(&mut self) {
        match self.selected_index {
            0 => self.config.exclude_hidden = !self.config.exclude_hidden,
            1 => self.config.exclude_symlinks = !self.config.exclude_symlinks,
            2 => self.config.exclude_temp_files = !self.config.exclude_temp_files,
            3 => {
                self.current_screen = Screen::ExcludeExtensions;
                self.selected_index = 0;
            }
            4 => {
                self.current_screen = Screen::ExcludeDirs;
                self.selected_index = 0;
            }
            5 => self.config.confirm_before_scan = !self.config.confirm_before_scan,
            _ => {}
        }
    }

    /// 로그 설정 메뉴 선택
    fn select_log_menu(&mut self) {
        match self.selected_index {
            0 => {
                self.start_input(InputTarget::EditLogPath, &self.config.log_path.clone());
            }
            1 => {
                if self.config.log_level == "info" {
                    self.config.log_level = "debug".to_string();
                } else {
                    self.config.log_level = "info".to_string();
                }
                self.status_message = format!("로그 레벨 변경: {}", self.config.log_level);
            }
            _ => {}
        }
    }

    /// 일반 설정 메뉴 선택
    fn select_general_menu(&mut self) {
        match self.selected_index {
            0 => self.config.recursive = !self.config.recursive,
            1 => {
                let current = self.config.scan_interval_seconds.to_string();
                self.start_input(InputTarget::EditScanInterval, &current);
            }
            2 => self.config.confirm_before_scan = !self.config.confirm_before_scan,
            _ => {}
        }
    }

    /// 텍스트 입력 모드 시작
    fn start_input(&mut self, target: InputTarget, prefill: &str) {
        self.input_mode = true;
        self.input_buffer = prefill.to_string();
        self.input_target = Some(target);
        self.status_message = "[Enter] 확인  [Esc] 취소".to_string();
    }

    /// 'a' 키 처리 (항목 추가)
    pub fn handle_add(&mut self) {
        match self.current_screen {
            Screen::WatchPaths => {
                self.current_screen = Screen::DirPicker;
                self.dir_picker = Some(DirPickerState::new());
                self.status_message = "[↑↓] 이동  [Enter] 폴더 진입  [Space] 선택/해제  [Tab] 포커스".to_string();
            }
            Screen::ExcludeExtensions => {
                self.start_input(InputTarget::AddExcludeExtension, ".");
            }
            Screen::ExcludeDirs => {
                self.start_input(InputTarget::AddExcludeDir, "");
            }
            _ => {}
        }
    }

    /// 'd' 키 처리 (항목 삭제)
    pub fn handle_delete(&mut self) {
        let has_items = match self.current_screen {
            Screen::WatchPaths => !self.config.watch_paths.is_empty(),
            Screen::ExcludeExtensions => !self.config.exclude_extensions.is_empty(),
            Screen::ExcludeDirs => !self.config.exclude_directories.is_empty(),
            _ => false,
        };
        if has_items {
            self.confirm_delete = true;
            self.delete_target_index = Some(self.selected_index);
            self.status_message = "정말 삭제하시겠습니까? [y/n]".to_string();
        }
    }

    /// 삭제 확인
    pub fn confirm_delete_yes(&mut self) {
        if let Some(idx) = self.delete_target_index {
            match self.current_screen {
                Screen::WatchPaths => {
                    if idx < self.config.watch_paths.len() {
                        let removed = self.config.watch_paths.remove(idx);
                        self.status_message = format!("삭제됨: {}", removed);
                    }
                }
                Screen::ExcludeExtensions => {
                    if idx < self.config.exclude_extensions.len() {
                        let removed = self.config.exclude_extensions.remove(idx);
                        self.status_message = format!("삭제됨: {}", removed);
                    }
                }
                Screen::ExcludeDirs => {
                    if idx < self.config.exclude_directories.len() {
                        let removed = self.config.exclude_directories.remove(idx);
                        self.status_message = format!("삭제됨: {}", removed);
                    }
                }
                _ => {}
            }
            // 인덱스 보정
            let max = self.menu_len();
            if max > 0 && self.selected_index >= max {
                self.selected_index = max - 1;
            }
        }
        self.confirm_delete = false;
        self.delete_target_index = None;
    }

    /// 텍스트 입력 제출 (Enter)
    pub fn submit_input(&mut self) {
        let value = self.input_buffer.trim().to_string();
        if value.is_empty() {
            self.cancel_input();
            return;
        }

        match &self.input_target {
            Some(InputTarget::AddExcludeExtension) => {
                let ext = if value.starts_with('.') {
                    value.clone()
                } else {
                    format!(".{}", value)
                };
                self.config.exclude_extensions.push(ext.clone());
                self.status_message = format!("추가됨: {}", ext);
            }
            Some(InputTarget::AddExcludeDir) => {
                self.config.exclude_directories.push(value.clone());
                self.status_message = format!("추가됨: {}", value);
            }
            Some(InputTarget::EditLogPath) => {
                let path = shellexpand::tilde(&value).to_string();
                self.config.log_path = path.clone();
                self.status_message = format!("로그 경로 변경: {}", path);
            }
            Some(InputTarget::EditScanInterval) => {
                if let Ok(seconds) = value.parse::<u64>() {
                    if seconds > 0 {
                        self.config.scan_interval_seconds = seconds;
                        self.status_message = format!("스캔 간격 변경: {}초", seconds);
                    } else {
                        self.status_message = "⚠️  1초 이상의 값을 입력해주세요.".to_string();
                    }
                } else {
                    self.status_message = "⚠️  숫자를 입력해주세요.".to_string();
                }
            }
            Some(InputTarget::DirPickerSearch) => {
                // 검색 종료 시 현재 위치 그대로 유지
                self.status_message = "[↑↓] 이동  [Enter] 폴더 진입  [Space] 선택/해제  [Tab] 포커스".to_string();
            }
            None => {}
        }

        self.input_mode = false;
        self.input_buffer.clear();
        self.input_target = None;
    }

    /// 텍스트 입력 취소 (Esc)
    pub fn cancel_input(&mut self) {
        let was_search = self.input_target == Some(InputTarget::DirPickerSearch);
        self.input_mode = false;
        self.input_buffer.clear();
        self.input_target = None;
        if was_search {
            self.status_message = "[↑↓] 이동  [Enter] 폴더 진입  [Space] 선택/해제  [Tab] 포커스".to_string();
        } else {
            self.status_message =
                "[↑↓] 이동  [Enter] 선택  [q] 저장 후 종료".to_string();
        }
    }

    /// 토글 (Space)
    pub fn toggle_current(&mut self) {
        match self.current_screen {
            Screen::ExcludeSettings => {
                // Enter와 동일하게 토글
                self.select();
            }
            Screen::GeneralSettings => {
                self.select();
            }
            _ => {}
        }
    }

    /// 뒤로 가기 (Esc, q)
    pub fn go_back(&mut self) {
        match self.current_screen {
            Screen::Main => {} // 메인에서는 아무것도 안 함
            Screen::ExcludeExtensions | Screen::ExcludeDirs => {
                self.current_screen = Screen::ExcludeSettings;
                self.selected_index = 0;
            }
            _ => {
                self.current_screen = Screen::Main;
                self.selected_index = 0;
            }
        }
        self.status_message =
            "[↑↓] 이동  [Enter] 선택  [q] 저장 후 종료".to_string();
    }

    /// DirPicker 공간/토글 처리
    pub fn toggle_dir_picker_selection(&mut self) {
        if let Some(dp) = &mut self.dir_picker {
            if dp.focus == DirPickerFocus::List {
                if let Some(path) = dp.items.get(dp.selected_index) {
                    // ".."은 선택할 수 없음
                    if path.file_name().map(|n| n == "..").unwrap_or(false) || path == &dp.current_dir.parent().unwrap_or(std::path::Path::new("")) {
                        return;
                    }
                    if dp.selected_paths.contains(path) {
                        dp.selected_paths.remove(path);
                    } else {
                        dp.selected_paths.insert(path.clone());
                    }
                }
            }
        }
    }

    /// DirPicker 선택 (Enter) 처리
    pub fn select_dir_picker(&mut self) {
        let mut transition = None;

        if let Some(dp) = &mut self.dir_picker {
            match dp.focus {
                DirPickerFocus::List => {
                    if let Some(path) = dp.items.get(dp.selected_index).cloned() {
                        dp.current_dir = path;
                        dp.refresh_items();
                    }
                }
                DirPickerFocus::Confirm => {
                    transition = Some(true); // 확인
                }
                DirPickerFocus::Cancel => {
                    transition = Some(false); // 취소
                }
            }
        }

        if let Some(confirm) = transition {
            if confirm {
                if let Some(dp) = self.dir_picker.take() {
                    let mut added = 0;
                    for path in dp.selected_paths {
                        let path_str = path.to_string_lossy().to_string();
                        if !self.config.watch_paths.contains(&path_str) {
                            self.config.watch_paths.push(path_str);
                            added += 1;
                        }
                    }
                    self.status_message = format!("{}개의 감시 폴더가 추가되었습니다.", added);
                }
            } else {
                self.dir_picker = None;
                self.status_message = "폴더 추가가 취소되었습니다.".to_string();
            }
            self.current_screen = Screen::WatchPaths;
            self.selected_index = 0;
        }
    }

    /// DirPicker 실시간 검색 매칭되는 모든 항목들의 인덱스 수집 (완전 일치 -> 시작 일치 -> 포함 일치 순서)
    pub fn get_matching_indices(&self) -> Vec<usize> {
        let query = self.input_buffer.to_lowercase();
        if query.is_empty() {
            return Vec::new();
        }

        let mut matches = Vec::new();
        if let Some(dp) = &self.dir_picker {
            // 1단계: 완전 일치
            for (idx, path) in dp.items.iter().enumerate() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name.to_lowercase() == query {
                        matches.push(idx);
                    }
                }
            }

            // 2단계: 시작 단어 일치 (완전 일치는 제외)
            for (idx, path) in dp.items.iter().enumerate() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    let name_lower = file_name.to_lowercase();
                    if name_lower.starts_with(&query) && name_lower != query {
                        matches.push(idx);
                    }
                }
            }

            // 3단계: 중간 포함 일치 (시작 일치, 완전 일치는 제외)
            for (idx, path) in dp.items.iter().enumerate() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    let name_lower = file_name.to_lowercase();
                    if name_lower.contains(&query) && !name_lower.starts_with(&query) {
                        matches.push(idx);
                    }
                }
            }
        }
        matches
    }

    /// DirPicker 실시간 검색 및 첫 번째 일치 항목으로 이동
    pub fn search_dir_picker(&mut self, _reset: bool) {
        let matches = self.get_matching_indices();
        if let Some(&first_match) = matches.first() {
            if let Some(dp) = &mut self.dir_picker {
                dp.selected_index = first_match;
                dp.list_state.select(Some(first_match));
            }
        }
    }

    /// 검색 결과 다음 매칭되는 폴더로 이동 (순환 지원)
    pub fn search_dir_picker_next(&mut self) {
        let matches = self.get_matching_indices();
        if matches.is_empty() {
            return;
        }

        if let Some(dp) = &mut self.dir_picker {
            let current = dp.selected_index;
            if let Some(pos) = matches.iter().position(|&x| x == current) {
                let next_pos = (pos + 1) % matches.len();
                let next_idx = matches[next_pos];
                dp.selected_index = next_idx;
                dp.list_state.select(Some(next_idx));
            } else {
                // 현재 선택 위치가 매칭 목록에 없으면 첫 번째 매칭으로 점프
                let next_idx = matches[0];
                dp.selected_index = next_idx;
                dp.list_state.select(Some(next_idx));
            }
        }
    }

    /// 검색 결과 이전 매칭되는 폴더로 이동 (순환 지원)
    pub fn search_dir_picker_prev(&mut self) {
        let matches = self.get_matching_indices();
        if matches.is_empty() {
            return;
        }

        if let Some(dp) = &mut self.dir_picker {
            let current = dp.selected_index;
            if let Some(pos) = matches.iter().position(|&x| x == current) {
                let prev_pos = if pos == 0 { matches.len() - 1 } else { pos - 1 };
                let prev_idx = matches[prev_pos];
                dp.selected_index = prev_idx;
                dp.list_state.select(Some(prev_idx));
            } else {
                // 현재 선택 위치가 매칭 목록에 없으면 마지막 매칭으로 점프
                let prev_idx = matches[matches.len() - 1];
                dp.selected_index = prev_idx;
                dp.list_state.select(Some(prev_idx));
            }
        }
    }

    /// DirPicker 검색 시작
    pub fn start_dir_picker_search(&mut self) {
        self.input_mode = true;
        self.input_buffer.clear();
        self.input_target = Some(InputTarget::DirPickerSearch);
        self.status_message = "[↑/↓] 매칭 순환  [Enter] 검색 완료  [Esc] 취소".to_string();
    }
}
