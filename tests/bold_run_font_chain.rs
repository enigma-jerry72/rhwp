//! bold run 이 'Noto Sans KR ExtraLight' 폴백으로
//! 떨어져 PDF 에서 제목/강조가 가늘게 렌더되는 문제의 회귀 가드.
//!
//! 인과: Task #1224 는 한컴 돋움 획 두께 정합을 위해 sans 폴백 체인의 무거운
//! Noto 직전에 ExtraLight 를 삽입했다 (본문 regular 대상). 그러나 bold run 도
//! 같은 체인을 쓰면 시스템 고딕 부재 환경(Linux/CI)에서 ExtraLight(200) 페이스가
//! 매칭된다. 브라우저는 faux-bold 를 합성해 화면은 굵게 보이지만 svg2pdf 는
//! 합성하지 않아 PDF 만 가늘어진다 (korea.kr 40쌍·hongbo 실측의 주 편차).
//!
//! 계약: 시각적 bold run 의 SVG font-family 체인에는 ExtraLight 가 없어야 하고,
//! regular run 의 체인은 Task #1224 그대로 ExtraLight 를 유지해야 한다.

use std::fs;
use std::path::Path;

const EXTRA_LIGHT: &str = "Noto Sans KR ExtraLight";

#[test]
fn bold_chain_skips_extralight_regular_keeps_it() {
    // regular: Task #1224 계약 유지
    let regular = rhwp::renderer::render_font_family_chain_weighted("돋움체", false);
    assert!(
        regular.contains(EXTRA_LIGHT),
        "regular 체인은 Task #1224 의 ExtraLight 를 유지해야 한다: {regular}"
    );
    // bold: ExtraLight 제외
    let bold = rhwp::renderer::render_font_family_chain_weighted("돋움체", true);
    assert!(
        !bold.contains(EXTRA_LIGHT),
        "bold 체인에 ExtraLight 가 있으면 시스템 고딕 부재 환경에서 굵기가 소실된다: {bold}"
    );
    // ExtraLight 제거 외에는 동일해야 한다 (여타 폴백 순서 불변)
    let stripped: String = regular
        .replace(&format!("'{EXTRA_LIGHT}',"), "")
        .replace(&format!(",'{EXTRA_LIGHT}'"), "");
    assert_eq!(
        bold, stripped,
        "bold 체인은 regular 체인에서 ExtraLight 만 뺀 것이어야 한다"
    );
}

/// 실문서 통합 가드: hongbo 보도자료 p1 의 bold <text> 노드에는 ExtraLight 가
/// 없고, 비-bold 노드에는 종전대로 남아 있다.
#[test]
fn hongbo_p1_bold_nodes_have_no_extralight() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join("samples/20250130-hongbo.hwp");
    let bytes = fs::read(&path).expect("read hongbo sample");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse");
    let svg = doc.render_page_svg_native(0).expect("render p1");

    let mut bold_nodes = 0usize;
    let mut bold_with_extralight = 0usize;
    let mut search_from = 0;
    while let Some(rel) = svg[search_from..].find("<text ") {
        let start = search_from + rel;
        let Some(close_rel) = svg[start..].find('>') else {
            break;
        };
        search_from = start + close_rel;
        let tag = &svg[start..start + close_rel];
        if tag.contains("font-weight=\"bold\"") {
            bold_nodes += 1;
            if tag.contains(EXTRA_LIGHT) {
                bold_with_extralight += 1;
            }
        }
    }

    assert!(
        bold_nodes > 0,
        "hongbo p1 에 bold run 이 있어야 한다 (fixture 확인)"
    );
    assert_eq!(
        bold_with_extralight, 0,
        "bold 노드 {bold_with_extralight}/{bold_nodes} 가 ExtraLight 체인을 갖고 있음 — 굵기 소실 재발"
    );
    // regular run 의 ExtraLight 유지(Task #1224)는 위 단위 테스트가 고정한다 —
    // 이 fixture 의 본문은 세리프 계열이라 sans ExtraLight 체인이 등장하지 않는다.
}
