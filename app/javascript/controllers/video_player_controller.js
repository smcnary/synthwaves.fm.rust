import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  static targets = ["video"]

  connect() {
    this.videoTarget.addEventListener("play", this.handlePlay.bind(this))

    // Pause video when audio player or YouTube starts
    document.addEventListener("player:play", this.pauseVideo.bind(this))
    document.addEventListener("player:playYouTube", this.pauseVideo.bind(this))
  }

  disconnect() {
    document.removeEventListener("player:play", this.pauseVideo.bind(this))
    document.removeEventListener("player:playYouTube", this.pauseVideo.bind(this))
  }

  handlePlay() {
    // Pause the persistent audio player when video plays
    const audio = document.getElementById("persistent-audio")
    if (audio && !audio.paused) audio.pause()
    document.dispatchEvent(new CustomEvent("youtube:stop"))
  }

  pauseVideo() {
    if (this.hasVideoTarget && !this.videoTarget.paused) {
      this.videoTarget.pause()
    }
  }
}
