import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  connect() {
    this.nativeHandler = window.webkit?.messageHandlers?.synthwaves
    if (!this.nativeHandler) return

    this.playNowHandler = (e) => this.forwardToNative("playNow", e)
    this.addHandler = (e) => this.forwardToNative("add", e)

    document.addEventListener("queue:playNow", this.playNowHandler, { capture: true })
    document.addEventListener("queue:add", this.addHandler, { capture: true })

    this.nowPlayingHandler = (e) => this.onNativeNowPlaying(e.detail)
    this.playbackStateHandler = (e) => this.onNativePlaybackState(e.detail)
    document.addEventListener("native:nowPlaying", this.nowPlayingHandler)
    document.addEventListener("native:playbackState", this.playbackStateHandler)
  }

  disconnect() {
    if (!this.nativeHandler) return

    document.removeEventListener("queue:playNow", this.playNowHandler, { capture: true })
    document.removeEventListener("queue:add", this.addHandler, { capture: true })
    document.removeEventListener("native:nowPlaying", this.nowPlayingHandler)
    document.removeEventListener("native:playbackState", this.playbackStateHandler)
  }

  forwardToNative(type, event) {
    event.stopImmediatePropagation()
    this.nativeHandler.postMessage({ type, payload: event.detail })
  }

  onNativeNowPlaying({ trackId }) {
    document.querySelectorAll("[data-song-row-track-id-value]").forEach((el) => {
      const isPlaying = el.dataset.songRowTrackIdValue === String(trackId)
      el.classList.toggle("now-playing", isPlaying)
    })
  }

  onNativePlaybackState({ state }) {
    document.dispatchEvent(new CustomEvent("player:nativeState", { detail: { state } }))
  }
}
