# Morgan Image Prep

Use LemonSlice for the avatar image.

## Hard requirements

- Host the final image at a public URL that returns `image/*`.
- Target a `368x560` export or any equivalent `9:14` portrait crop.
- Keep Morgan’s face and mouth clearly visible.
- Avoid anything that covers the jawline, lips, or eyes.

## Best composition

- Head-and-shoulders portrait.
- Face mostly forward or a very slight three-quarter angle.
- Leave crop safety above the head and below the shoulders so LemonSlice center-cropping does not cut off the face.
- Neutral or lightly friendly expression.
- Soft, even lighting with clear facial detail.
- Simple background with contrast against the subject.

## What to avoid

- Side profiles.
- Heavy shadows across the face.
- Hands, microphones, or props near the mouth.
- Sunglasses, hair, or hats obscuring the eyes.
- Tiny subject in a large frame.
- Busy or high-contrast backgrounds that distract from the face.

## Generator prompt starter

```text
Photorealistic head-and-shoulders portrait of Morgan, centered composition, facing camera, neutral-friendly expression, mouth clearly visible, eyes clearly visible, soft even studio lighting, clean simple background, realistic skin detail, natural proportions, upper torso visible, no hands near face, no sunglasses, no harsh shadows, portrait orientation, 9:14 aspect ratio.
```

## Hosting notes

- If Morgan is already uploaded in LemonSlice, prefer `MORGAN_LEMONSLICE_AGENT_ID` over public image hosting.
- If the final Morgan portrait is not ready yet, use `MORGAN_PLACEHOLDER_IMAGE_URL`.
- To force the placeholder while wiring the backend, set `MORGAN_USE_PLACEHOLDER_IMAGE=true`.
- Keep both the real image and placeholder accessible over HTTPS so the backend can switch instantly during testing.
