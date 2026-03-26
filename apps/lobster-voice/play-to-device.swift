import AVFoundation
import Foundation

guard CommandLine.arguments.count >= 3,
      let deviceID = UInt32(CommandLine.arguments[2]) else {
    fputs("Usage: play-to-device <audio-file> <coreaudio-device-id>\n", stderr)
    fputs("Device IDs: `say --audio-device '?'`\n", stderr)
    exit(1)
}

let url = URL(fileURLWithPath: CommandLine.arguments[1])
guard FileManager.default.fileExists(atPath: url.path) else {
    fputs("play-to-device: file not found: \(url.path)\n", stderr)
    exit(1)
}

let engine = AVAudioEngine()
let player = AVAudioPlayerNode()
engine.attach(player)

var devID = AudioDeviceID(deviceID)
let status = AudioUnitSetProperty(
    engine.outputNode.audioUnit!,
    kAudioOutputUnitProperty_CurrentDevice,
    kAudioUnitScope_Global,
    0,
    &devID,
    UInt32(MemoryLayout<AudioDeviceID>.size)
)
guard status == noErr else {
    fputs("play-to-device: failed to set output device \(deviceID) (err \(status))\n", stderr)
    exit(1)
}

do {
    let file = try AVAudioFile(forReading: url)
    engine.connect(player, to: engine.mainMixerNode, format: file.processingFormat)
    try engine.start()
    let done = DispatchSemaphore(value: 0)
    player.scheduleFile(file, at: nil) { done.signal() }
    player.play()
    done.wait()
    Thread.sleep(forTimeInterval: 0.15)
} catch {
    fputs("play-to-device: \(error.localizedDescription)\n", stderr)
    exit(1)
}
