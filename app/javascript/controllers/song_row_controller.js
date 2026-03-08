import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  static values = { trackId: Number, title: String, artist: String, streamUrl: String }

  connect() {
    console.log("[song-row] connected", { trackId: this.trackIdValue, element: this.element })
  }

  play(event) {
    event.preventDefault()
    event.stopPropagation()
    console.log("[song-row] play() fired", { trackId: this.trackIdValue, streamUrl: this.streamUrlValue })
    const track = {
      trackId: this.trackIdValue,
      title: this.titleValue,
      artist: this.artistValue,
      streamUrl: this.streamUrlValue
    }
    document.dispatchEvent(new CustomEvent("queue:playNow", { detail: track }))
  }

  addToQueue() {
    const track = {
      trackId: this.trackIdValue,
      title: this.titleValue,
      artist: this.artistValue,
      streamUrl: this.streamUrlValue
    }
    document.dispatchEvent(new CustomEvent("queue:add", { detail: track }))
  }
}
