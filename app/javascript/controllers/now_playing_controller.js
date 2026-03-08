import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  static values = { trackId: Number }

  connect() {
    this.nowPlayingHandler = (e) => this.onNowPlaying(e.detail)
    document.addEventListener("player:nowPlaying", this.nowPlayingHandler)
    this.checkCurrentTrack()
  }

  disconnect() {
    document.removeEventListener("player:nowPlaying", this.nowPlayingHandler)
  }

  onNowPlaying({ trackId }) {
    this.element.dataset.nowPlaying = (trackId === this.trackIdValue).toString()
  }

  checkCurrentTrack() {
    try {
      const saved = localStorage.getItem("playerCurrentTrack")
      if (saved) {
        const track = JSON.parse(saved)
        this.element.dataset.nowPlaying = (track.trackId === this.trackIdValue).toString()
      }
    } catch (e) {
      // Ignore invalid data
    }
  }
}
