- ● The baseline builds. The current font is an 8×8 bitmap font scaled 2–3× with nearest-neighbor filtering — that's the blockiness. I'll replace it
  with the Hack Nerd Font TTF, rasterized anti-aliased at the exact pixel sizes the UI uses.
  - 기초가 다져집니다. 현재 사용 중인 폰트는 8×8 비트맵 폰트로, 2~3배 크기로 확대되고 최근접 이웃 필터링이 적용되어 있습니다 — 그래서 블록이 뭉개져 보이는 거죠. 내가 대신할게
  Hack Nerd 폰트 TTF를 사용하면, UI에서 사용하는 정확한 픽셀 크기로 래스터화된 안티앨리어싱이 적용됩니다. 

- Rasterizing TTF font… (8m 28s · ↓ 30.8k tokens)
```bash
  ⎿  ◼ Add ab_glyph and rasterize TTF font
     ◻ Rebuild glyph atlas with per-glyph quads
     ◻ Update UI text rendering and layout
     ◻ Verify: tests + visual frame dump
```

- "This is a 4x nearest-neighbor zoomed crop of
  text rendered in a GPU app. Look closely at the glyph edges. 
  - Question 1: Do the letter edges show smooth multi-level anti-aliasing (grayscale intermediate pixels forming curved strokes), or hard binary edges with stair-stepping (pure on/off pixels, jagged diagonals like classic bitmap
  fonts)?
  - Question 2: What font does this appear to be (it should look like 'Hack' with slightly boxy curves)?
  - Question 3: Does the letter 'o' or 'D' show smooth rounded curves? Be specific about whether intermediate gray pixels exist at edges.
