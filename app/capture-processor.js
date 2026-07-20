class CaptureProcessor extends AudioWorkletProcessor {
  process(inputs) {
    const channel = inputs[0] && inputs[0][0];
    if (channel && channel.length > 0) {
      // Copy the block: the underlying buffer is reused by the audio
      // render thread on the next call, so it can't be posted as-is.
      this.port.postMessage(channel.slice(0));
    }
    return true;
  }
}

registerProcessor("capture-processor", CaptureProcessor);
