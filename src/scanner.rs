// ============================================================================
// stick - 파일명 NFD→NFC 변환 스캐너 모듈
// 핵심 로직: macOS NFD 한글 파일명을 NFC로 정규화
// ============================================================================

use anyhow::{Context, Result};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info};
use unicode_normalization::UnicodeNormalization;
use walkdir::WalkDir;

use crate::config::StickConfig;

// ── 스캔 결과 구조체 ─────────────────────────────────────────────────────

/// 단일 파일/디렉토리의 변환 결과
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RenameEntry {
    /// 원본 경로
    pub original_path: PathBuf,
    /// 변환 후 경로
    pub new_path: PathBuf,
    /// 원본 파일명 (표시용)
    pub original_name: String,
    /// 변환 후 파일명 (표시용)
    pub new_name: String,
}

/// 전체 스캔 결과 집계
#[derive(Debug, Default)]
pub struct ScanResult {
    /// 성공적으로 변환된 항목
    pub renamed: Vec<RenameEntry>,
    /// 제외 규칙에 의해 스킵된 항목 수
    pub skipped_count: usize,
    /// 에러 발생 항목
    pub errors: Vec<String>,
    /// 전체 검사 항목 수
    pub total_scanned: usize,
}

impl ScanResult {
    /// 결과 요약 문자열 반환
    pub fn summary(&self) -> String {
        format!(
            "스캔 완료: 전체 {}건 검사, 변환 {}건, 스킵 {}건, 에러 {}건",
            self.total_scanned,
            self.renamed.len(),
            self.skipped_count,
            self.errors.len()
        )
    }
}

// ── 파일시스템 유틸리티 ──────────────────────────────────────────────────

/// 두 경로가 같은 파일(inode)을 가리키는지 확인
/// macOS의 APFS/HFS+에서는 NFD와 NFC 경로가 동일한 파일을 참조합니다.
#[cfg(unix)]
fn same_inode(path_a: &Path, path_b: &Path) -> bool {
    let meta_a = match fs::metadata(path_a) {
        Ok(m) => m,
        Err(_) => return false,
    };
    let meta_b = match fs::metadata(path_b) {
        Ok(m) => m,
        Err(_) => return false,
    };
    // 같은 디바이스의 같은 inode이면 동일 파일
    meta_a.dev() == meta_b.dev() && meta_a.ino() == meta_b.ino()
}

/// Windows/기타 OS용 두 경로의 동일성 판단
/// canonicalize를 통해 물리적 절대 경로로 가공하여 정밀 비교합니다.
#[cfg(not(unix))]
fn same_inode(path_a: &Path, path_b: &Path) -> bool {
    if let (Ok(canon_a), Ok(canon_b)) = (fs::canonicalize(path_a), fs::canonicalize(path_b)) {
        canon_a == canon_b
    } else {
        false
    }
}

/// 디스크에 저장된 실제 파일명을 반환합니다.
/// FSEvents는 NFC 파일이라도 NFD 경로로 이벤트를 발생시킬 수 있으므로, inode 비교를 통해 실제 이름을 찾습니다.
#[cfg(unix)]
fn get_real_filename(path: &Path) -> Option<String> {
    let parent = path.parent()?;
    if let Ok(entries) = std::fs::read_dir(parent) {
        for entry in entries.flatten() {
            if same_inode(path, &entry.path()) {
                return entry.file_name().into_string().ok();
            }
        }
    }
    None
}

// ── 핵심 변환 함수 ───────────────────────────────────────────────────────

/// 문자열이 NFD 형태인지 확인
/// NFC로 변환한 결과가 원본과 다르면 NFD 형태로 판단합니다.
pub fn is_nfd_string(text: &str) -> bool {
    let nfc_normalized: String = text.nfc().collect();
    nfc_normalized != text
}

/// 문자열을 NFC 형태로 변환
pub fn to_nfc(text: &str) -> String {
    text.nfc().collect()
}

/// 단일 경로의 파일/디렉토리명을 NFC로 변환
/// 변환이 필요없으면 None, 변환되면 Some(RenameEntry)를 반환합니다.
pub fn normalize_path(path: &Path, dry_run: bool) -> Result<Option<RenameEntry>> {
    // 파일명 추출 (경로의 마지막 컴포넌트)
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .context("파일명을 읽을 수 없습니다")?;

    // NFD 여부 확인 - NFC와 같으면 변환 불필요
    if !is_nfd_string(file_name) {
        return Ok(None);
    }

    // NFC로 변환된 새 파일명 생성
    let mut nfc_name = to_nfc(file_name);
    let mut new_path = path.with_file_name(&nfc_name);

    // 이름 충돌 방지
    // macOS의 APFS/HFS+는 NFD/NFC를 동일 파일로 취급하므로,
    // 실제로 다른 파일이 존재하는 경우만 충돌로 처리합니다.
    if new_path.exists() {
        // 같은 파일인지 확인 (inode 비교)
        let is_same_file = same_inode(path, &new_path);
        if !is_same_file {
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or(&nfc_name);
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            
            let mut counter = 1;
            loop {
                let conflict_name = if ext.is_empty() {
                    format!("{}_conflict{}", to_nfc(stem), if counter > 1 { format!("_{}", counter) } else { "".to_string() })
                } else {
                    format!("{}_conflict{}.{}", to_nfc(stem), if counter > 1 { format!("_{}", counter) } else { "".to_string() }, ext)
                };
                
                let candidate_path = path.with_file_name(&conflict_name);
                if !candidate_path.exists() {
                    nfc_name = conflict_name;
                    new_path = candidate_path;
                    break;
                }
                counter += 1;
            }
            tracing::warn!("파일 이름 충돌 감지! 충돌 방지명 적용: {}", new_path.display());
        }
        // 같은 파일이면 rename 진행 (OS가 안전하게 처리)
    }

    let entry = RenameEntry {
        original_path: path.to_path_buf(),
        new_path: new_path.clone(),
        original_name: file_name.to_string(),
        new_name: nfc_name,
    };

    // dry_run이 아닌 경우에만 실제 rename 수행
    if !dry_run {
        fs::rename(path, &new_path).with_context(|| {
            format!(
                "파일명 변경 실패: {} → {}",
                path.display(),
                new_path.display()
            )
        })?;
        info!(
            "[RENAME] {} → {} (NFD→NFC)",
            entry.original_name, entry.new_name
        );
    } else {
        info!(
            "[미리보기] {} → {} (변환 예정)",
            entry.original_name, entry.new_name
        );
    }

    Ok(Some(entry))
}

// ── 제외 규칙 판단 ───────────────────────────────────────────────────────

/// 경로가 제외 대상인지 판단
/// 숨김파일, 심볼릭 링크, 임시파일, 확장자, 디렉토리명 등을 확인합니다.
fn should_exclude(path: &Path, config: &StickConfig) -> bool {
    let file_name = match path.file_name().and_then(|n| n.to_str()) {
        Some(name) => name,
        None => return true, // 파일명을 읽을 수 없으면 제외
    };

    // 숨김 파일 제외 (. 으로 시작)
    if config.exclude_hidden && file_name.starts_with('.') {
        debug!("[SKIP] 숨김 파일/폴더 제외: {}", file_name);
        return true;
    }

    // 심볼릭 링크 제외
    if config.exclude_symlinks && path.is_symlink() {
        debug!("[SKIP] 심볼릭 링크 제외: {}", file_name);
        return true;
    }

    // 임시 파일 제외 (~ 로 끝나는 파일)
    if config.exclude_temp_files && file_name.ends_with('~') {
        debug!("[SKIP] 임시 파일 제외: {}", file_name);
        return true;
    }

    // 확장자 기반 제외
    if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
        let dot_extension = format!(".{}", extension);
        if config
            .exclude_extensions
            .iter()
            .any(|ext| ext.eq_ignore_ascii_case(&dot_extension))
        {
            debug!("[SKIP] 확장자 제외: {} ({})", file_name, dot_extension);
            return true;
        }
    }

    // 디렉토리명 기반 제외 (정확히 일치하는 이름만)
    if path.is_dir()
        && config
            .exclude_directories
            .iter()
            .any(|dir| dir == file_name)
    {
        debug!("[SKIP] 제외 폴더: {}", file_name);
        return true;
    }

    // 경로의 어떤 컴포넌트가 제외 디렉토리에 해당하는지 확인
    for component in path.components() {
        if let std::path::Component::Normal(os_str) = component {
            if let Some(comp_name) = os_str.to_str() {
                if config.exclude_directories.iter().any(|dir| dir == comp_name) {
                    debug!("[SKIP] 제외 폴더 내부 경로: {}", path.display());
                    return true;
                }
            }
        }
    }

    false
}

// ── 디렉토리 스캔 ────────────────────────────────────────────────────────

/// 지정된 디렉토리를 스캔하여 NFD 파일명을 찾고 NFC로 변환
///
/// **중요**: 디렉토리 변환 시 반드시 하위(leaf)부터 상위(root) 순서로 처리합니다.
/// 상위 디렉토리를 먼저 rename하면 하위 경로가 무효화되기 때문입니다.
pub fn scan_directory(path: &Path, config: &StickConfig, dry_run: bool) -> Result<ScanResult> {
    let mut result = ScanResult::default();

    if !path.exists() {
        return Err(anyhow::anyhow!("경로가 존재하지 않습니다: {}", path.display()));
    }

    if !path.is_dir() {
        return Err(anyhow::anyhow!("디렉토리가 아닙니다: {}", path.display()));
    }

    // walkdir로 모든 항목 수집
    let walker = if config.recursive {
        WalkDir::new(path).follow_links(false)
    } else {
        WalkDir::new(path).max_depth(1).follow_links(false)
    };

    // 모든 항목을 수집 후, 깊이(depth) 역순으로 정렬
    // 이렇게 하면 하위 → 상위 순서로 처리됩니다.
    let mut entries: Vec<_> = walker
        .into_iter()
        .filter_entry(|e| !should_exclude(e.path(), config))
        .filter_map(|entry| entry.ok())
        .collect();

    // 깊이 역순 정렬 (하위 → 상위)
    entries.sort_by(|a, b| b.depth().cmp(&a.depth()));

    for entry in entries {
        let entry_path = entry.path();

        // 루트 디렉토리 자체는 스킵
        if entry_path == path {
            continue;
        }

        result.total_scanned += 1;

        // 제외 규칙 확인
        if should_exclude(entry_path, config) {
            result.skipped_count += 1;
            continue;
        }

        // NFD → NFC 변환 시도
        match normalize_path(entry_path, dry_run) {
            Ok(Some(rename_entry)) => {
                result.renamed.push(rename_entry);
            }
            Ok(None) => {
                // 이미 NFC 형태 → 변환 불필요
            }
            Err(err) => {
                let error_msg = format!("{}: {}", entry_path.display(), err);
                error!("[ERROR] {}", error_msg);
                result.errors.push(error_msg);
            }
        }
    }

    debug!("{}", result.summary());
    Ok(result)
}

/// 단일 파일 경로에 대해 NFC 변환 수행 (watcher에서 호출용)
/// FSEvents가 제공하는 경로는 항상 NFD 형태일 수 있으므로(실제 디스크가 NFC라도), 
/// 실제 디스크의 파일명을 찾아 무한 루프를 방지합니다.
pub fn normalize_single_path(path: &Path, config: &StickConfig) -> Result<Option<RenameEntry>> {
    if should_exclude(path, config) {
        return Ok(None);
    }
    
    #[cfg(unix)]
    let real_name = get_real_filename(path);
    #[cfg(not(unix))]
    let real_name = path.file_name().and_then(|n| n.to_str()).map(|s| s.to_string());

    if let Some(real_name_str) = real_name {
        if !is_nfd_string(&real_name_str) {
            // 실제 디스크의 이름이 이미 NFC라면 무시 (무한 루프 방지)
            return Ok(None);
        }
        
        // 실제 경로 기준으로 변환 수행
        let real_path = path.with_file_name(&real_name_str);
        return normalize_path(&real_path, false);
    }

    // fallback
    normalize_path(path, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_nfd_string() {
        // NFC 형태 "가" (U+AC00)
        let nfc = "\u{AC00}";
        assert!(!is_nfd_string(nfc));

        // NFD 형태 "가" (ᄀ U+1100 + ᅡ U+1161)
        let nfd = "\u{1100}\u{1161}";
        assert!(is_nfd_string(nfd));

        // 영문은 NFC/NFD 동일
        assert!(!is_nfd_string("hello.txt"));
    }

    #[test]
    fn test_to_nfc() {
        // NFD "가" → NFC "가"
        let nfd = "\u{1100}\u{1161}";
        let nfc = to_nfc(nfd);
        assert_eq!(nfc, "\u{AC00}");
    }

    #[test]
    fn test_ascii_unchanged() {
        let ascii = "document.pdf";
        assert_eq!(to_nfc(ascii), ascii);
        assert!(!is_nfd_string(ascii));
    }
}
