//! svg2pdf 폰트 서브셋 실패(SubsetError)로 export-pdf 가
//! 문서 단위로 통째 실패하던 문제의 회귀 가드.
//!
//! 재현: `samples/pua-test.hwp` (1쪽, PUA/희귀 글리프 포함) —
//! `SVG→chunk 변환 실패: SubsetError(ID(InnerId(215v1)))`. 저장소 197쌍 스윕에서
//! 동일 원인 5건 (inner-table-01, kps-ai, mel-001 등). 정황: TTF 폴백에 글리프
//! 부재 → CFF 포맷(Noto CJK TTC) 폴백 → subsetter 0.2.6 CFF 서브셋 실패.
//!
//! 계약 (수정 후): 서브셋이 실패한 페이지는 글리프를 path 로 변환(embed_text=false,
//! Task #2264 기전)해 재시도한다 — 해당 페이지 텍스트 추출만 비활성화되고
//! 문서 전체 export 는 성공해야 한다.

use std::fs;
use std::path::Path;

#[test]
fn pua_document_exports_pdf_despite_subset_error() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join("samples/pua-test.hwp");
    let bytes = fs::read(&path).expect("read pua-test sample");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse");

    let svg = doc.render_page_svg_native(0).expect("render p1 svg");
    let pdf = rhwp::renderer::pdf::svgs_to_pdf(&[svg])
        .expect("서브셋 실패 페이지는 path 폴백으로 export 되어야 한다");

    // 유효한 PDF 인지 최소 확인
    assert!(pdf.starts_with(b"%PDF-"), "PDF 헤더가 있어야 한다");
    assert!(
        pdf.len() > 1000,
        "본문이 있는 PDF 여야 한다 (len={})",
        pdf.len()
    );
}
