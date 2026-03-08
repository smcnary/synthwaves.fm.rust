import { Controller } from "@hotwired/stimulus"

// Manages the YouTube IFrame Player API and PiP video window.
// Listens for "youtube:play" events and controls embedded YouTube playback.
export default class extends Controller {
  static targets = ["container", "iframe", "closeButton"]

  connect() {
    this.player = null
    this.apiReady = false
    this.pendingVideoId = null
    this.pendingIsLive = false
    this.currentIsLive = false
    this.timeUpdateInterval = null

    this.playHandler = (e) => this.play(e.detail)
    this.stopHandler = () => this.stop()
    this.toggleHandler = () => this.toggle()

    document.addEventListener("youtube:play", this.playHandler)
    document.addEventListener("youtube:stop", this.stopHandler)
    document.addEventListener("youtube:toggle", this.toggleHandler)
  }

  disconnect() {
    document.removeEventListener("youtube:play", this.playHandler)
    document.removeEventListener("youtube:stop", this.stopHandler)
    document.removeEventListener("youtube:toggle", this.toggleHandler)
    this.stopTimeUpdates()
    if (this.player) {
      this.player.destroy()
      this.player = null
    }
  }

  play({ videoId, isLive }) {
    this.currentIsLive = isLive || false

    if (!this.apiReady) {
      this.pendingVideoId = videoId
      this.pendingIsLive = isLive || false
      this.loadApi()
      return
    }

    if (this.player) {
      this.player.loadVideoById(videoId)
    } else {
      this.createPlayer(videoId)
    }

    this.show()
  }

  toggle() {
    if (!this.player) return

    const state = this.player.getPlayerState()
    if (state === YT.PlayerState.PLAYING) {
      this.player.pauseVideo()
    } else {
      this.player.playVideo()
    }
  }

  stop() {
    if (this.player) {
      this.player.stopVideo()
    }
    this.stopTimeUpdates()
    this.hide()
    document.dispatchEvent(new CustomEvent("youtube:stopped"))
  }

  close() {
    this.stop()
  }

  // Private

  loadApi() {
    if (window.YT && window.YT.Player) {
      this.apiReady = true
      this.onApiReady()
      return
    }

    if (document.querySelector('script[src*="youtube.com/iframe_api"]')) {
      // Script already loading, wait for callback
      const originalCallback = window.onYouTubeIframeAPIReady
      window.onYouTubeIframeAPIReady = () => {
        if (originalCallback) originalCallback()
        this.apiReady = true
        this.onApiReady()
      }
      return
    }

    window.onYouTubeIframeAPIReady = () => {
      this.apiReady = true
      this.onApiReady()
    }

    const script = document.createElement("script")
    script.src = "https://www.youtube.com/iframe_api"
    document.head.appendChild(script)
  }

  onApiReady() {
    if (this.pendingVideoId) {
      const videoId = this.pendingVideoId
      const isLive = this.pendingIsLive
      this.pendingVideoId = null
      this.pendingIsLive = false
      this.currentIsLive = isLive
      this.createPlayer(videoId)
      this.show()
    }
  }

  createPlayer(videoId) {
    if (this.player) {
      this.player.destroy()
      this.player = null
    }

    this.player = new YT.Player(this.iframeTarget, {
      height: "144",
      width: "256",
      videoId: videoId,
      playerVars: {
        autoplay: 1,
        controls: 0,
        modestbranding: 1,
        rel: 0,
        playsinline: 1
      },
      events: {
        onReady: () => this.onPlayerReady(),
        onStateChange: (e) => this.onStateChange(e)
      }
    })
  }

  onPlayerReady() {
    this.player.playVideo()
  }

  onStateChange(event) {
    switch (event.data) {
      case YT.PlayerState.PLAYING:
        document.dispatchEvent(new CustomEvent("youtube:stateChange", {
          detail: { state: "playing" }
        }))
        if (!this.currentIsLive) {
          this.startTimeUpdates()
        }
        break
      case YT.PlayerState.PAUSED:
        document.dispatchEvent(new CustomEvent("youtube:stateChange", {
          detail: { state: "paused" }
        }))
        this.stopTimeUpdates()
        break
      case YT.PlayerState.ENDED:
        this.stopTimeUpdates()
        document.dispatchEvent(new CustomEvent("youtube:stateChange", {
          detail: { state: "ended" }
        }))
        break
    }
  }

  startTimeUpdates() {
    this.stopTimeUpdates()
    this.timeUpdateInterval = setInterval(() => {
      if (this.player && this.player.getCurrentTime) {
        document.dispatchEvent(new CustomEvent("youtube:timeUpdate", {
          detail: {
            currentTime: this.player.getCurrentTime(),
            duration: this.player.getDuration()
          }
        }))
      }
    }, 500)
  }

  stopTimeUpdates() {
    if (this.timeUpdateInterval) {
      clearInterval(this.timeUpdateInterval)
      this.timeUpdateInterval = null
    }
  }

  show() {
    this.containerTarget.classList.remove("hidden")
  }

  hide() {
    this.containerTarget.classList.add("hidden")
  }
}
