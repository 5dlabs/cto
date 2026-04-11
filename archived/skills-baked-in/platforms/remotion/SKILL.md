---
name: remotion
description: Programmatic video creation with React - animations, assets, captions, transitions, charts.
---

# Remotion Best Practices

Programmatic video creation using React components.

## When to Use

Use this skill when dealing with Remotion code for domain-specific knowledge on:
- Video composition and sequencing
- Animations and transitions
- Audio and video asset handling
- Captions and subtitles
- Charts and data visualization
- 3D content with Three.js/R3F

## Core Concepts

### Compositions

```tsx
import { Composition } from 'remotion';

export const Root = () => {
  return (
    <Composition
      id="MyVideo"
      component={MyVideo}
      durationInFrames={150}
      fps={30}
      width={1920}
      height={1080}
    />
  );
};
```

### Basic Animation

```tsx
import { useCurrentFrame, interpolate } from 'remotion';

export const MyAnimation = () => {
  const frame = useCurrentFrame();
  
  const opacity = interpolate(frame, [0, 30], [0, 1], {
    extrapolateRight: 'clamp',
  });
  
  const scale = interpolate(frame, [0, 30], [0.5, 1]);
  
  return (
    <div style={{ 
      opacity, 
      transform: `scale(${scale})` 
    }}>
      Hello World
    </div>
  );
};
```

### Sequencing

```tsx
import { Sequence, useVideoConfig } from 'remotion';

export const MyVideo = () => {
  const { fps } = useVideoConfig();
  
  return (
    <>
      <Sequence from={0} durationInFrames={fps * 2}>
        <Intro />
      </Sequence>
      <Sequence from={fps * 2} durationInFrames={fps * 5}>
        <MainContent />
      </Sequence>
      <Sequence from={fps * 7}>
        <Outro />
      </Sequence>
    </>
  );
};
```

### Video and Audio

```tsx
import { Video, Audio, OffthreadVideo } from 'remotion';

export const MediaExample = () => {
  return (
    <>
      {/* Use OffthreadVideo for better performance */}
      <OffthreadVideo src={staticFile('background.mp4')} />
      
      <Audio
        src={staticFile('music.mp3')}
        volume={0.5}
        startFrom={30}  // Start from frame 30
      />
    </>
  );
};
```

### Spring Animations

```tsx
import { spring, useCurrentFrame, useVideoConfig } from 'remotion';

export const SpringAnimation = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  
  const scale = spring({
    frame,
    fps,
    config: {
      damping: 10,
      stiffness: 100,
      mass: 0.5,
    },
  });
  
  return <div style={{ transform: `scale(${scale})` }}>Bouncy!</div>;
};
```

### Text Animations

```tsx
import { useCurrentFrame, interpolate } from 'remotion';

export const TypewriterText = ({ text }: { text: string }) => {
  const frame = useCurrentFrame();
  const charsToShow = Math.floor(frame / 2);
  
  return <span>{text.slice(0, charsToShow)}</span>;
};
```

### Captions

```tsx
import { useCurrentFrame, useVideoConfig } from 'remotion';

interface Caption {
  text: string;
  startFrame: number;
  endFrame: number;
}

export const Captions = ({ captions }: { captions: Caption[] }) => {
  const frame = useCurrentFrame();
  
  const currentCaption = captions.find(
    c => frame >= c.startFrame && frame < c.endFrame
  );
  
  return currentCaption ? (
    <div className="caption">{currentCaption.text}</div>
  ) : null;
};
```

### Transitions

```tsx
import { TransitionSeries, linearTiming, fade } from '@remotion/transitions';

export const TransitionExample = () => {
  return (
    <TransitionSeries>
      <TransitionSeries.Sequence durationInFrames={60}>
        <SlideOne />
      </TransitionSeries.Sequence>
      <TransitionSeries.Transition
        timing={linearTiming({ durationInFrames: 30 })}
        presentation={fade()}
      />
      <TransitionSeries.Sequence durationInFrames={60}>
        <SlideTwo />
      </TransitionSeries.Sequence>
    </TransitionSeries>
  );
};
```

## Key Topics Reference

| Topic | What It Covers |
|-------|---------------|
| animations | Fundamental animation skills |
| assets | Importing images, videos, audio, fonts |
| audio | Sound - importing, trimming, volume, speed, pitch |
| calculate-metadata | Dynamic duration, dimensions, props |
| compositions | Defining compositions, stills, folders |
| display-captions | TikTok-style captions with word highlighting |
| fonts | Loading Google Fonts and local fonts |
| gifs | GIFs synchronized with timeline |
| images | Embedding images with Img component |
| lottie | Lottie animations |
| sequencing | Delay, trim, limit duration |
| tailwind | Using TailwindCSS |
| text-animations | Typography and text animation |
| timing | Interpolation curves - linear, easing, spring |
| transitions | Scene transition patterns |
| trimming | Cut beginning/end of animations |
| videos | Embedding videos - trimming, volume, speed, looping |
| 3d | Three.js and React Three Fiber |
| charts | Data visualization |

## Best Practices

1. **Use `staticFile()` for assets** - Ensures proper bundling
2. **Prefer `OffthreadVideo` over `Video`** - Better performance
3. **Use spring animations** - More natural motion
4. **Calculate metadata dynamically** - For data-driven videos
5. **Use sequences for organization** - Keep timeline manageable
6. **Test with `npm run preview`** - Fast iteration

## Attribution

Based on [remotion-dev/skills](https://github.com/remotion-dev/skills) remotion-best-practices - 220+ installs.
