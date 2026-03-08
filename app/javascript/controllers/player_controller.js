import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  static targets = ["audio", "progress", "title", "artist", "artwork", "playIcon", "pauseIcon", "currentTime", "duration", "volume", "progressBar", "liveIndicator", "prevButton", "nextButton"]
  static values = { playHistoryUrl: String }

  connect() {
    this.audio = this.audioTarget
    this.youtubeActive = false
    this.youtubePlaying = false
    this.currentIsLive = false

    this.audio.addEventListener("timeupdate", () => this.onTimeUpdate())
    this.audio.addEventListener("ended", () => this.onEnded())
    this.audio.addEventListener("loadedmetadata", () => this.onLoadedMetadata())
    this.audio.addEventListener("play", () => this.updatePlayPauseIcon())
    this.audio.addEventListener("pause", () => this.updatePlayPauseIcon())

    this.playTrackHandler = (e) => this.playTrack(e.detail)
    this.playYouTubeHandler = (e) => this.playYouTube(e.detail)
    this.youtubeStateHandler = (e) => this.onYouTubeState(e.detail)
    this.youtubeTimeHandler = (e) => this.onYouTubeTime(e.detail)
    this.youtubeStoppedHandler = () => this.onYouTubeStopped()

    document.addEventListener("player:play", this.playTrackHandler)
    document.addEventListener("player:playYouTube", this.playYouTubeHandler)
    document.addEventListener("youtube:stateChange", this.youtubeStateHandler)
    document.addEventListener("youtube:timeUpdate", this.youtubeTimeHandler)
    document.addEventListener("youtube:stopped", this.youtubeStoppedHandler)

    if ("mediaSession" in navigator) {
      navigator.mediaSession.setActionHandler("play", () => this.toggle())
      navigator.mediaSession.setActionHandler("pause", () => this.toggle())
      navigator.mediaSession.setActionHandler("previoustrack", () => this.previous())
      navigator.mediaSession.setActionHandler("nexttrack", () => this.next())
    }
  }

  disconnect() {
    document.removeEventListener("player:play", this.playTrackHandler)
    document.removeEventListener("player:playYouTube", this.playYouTubeHandler)
    document.removeEventListener("youtube:stateChange", this.youtubeStateHandler)
    document.removeEventListener("youtube:timeUpdate", this.youtubeTimeHandler)
    document.removeEventListener("youtube:stopped", this.youtubeStoppedHandler)
  }

  playTrack({ trackId, title, artist, streamUrl }) {
    // Switch from YouTube to local
    if (this.youtubeActive) {
      document.dispatchEvent(new CustomEvent("youtube:stop"))
      this.youtubeActive = false
      this.youtubePlaying = false
    }

    this.currentIsLive = false
    this.currentTrackId = trackId
    this.titleTarget.textContent = title
    this.artistTarget.textContent = artist
    this.showNormalMode()

    this.audio.src = streamUrl
    this.audio.play()

    if ("mediaSession" in navigator) {
      navigator.mediaSession.metadata = new MediaMetadata({ title, artist })
    }

    this.recordPlay(trackId)
  }

  playYouTube({ trackId, title, artist, youtubeVideoId, isLive }) {
    // Pause local audio
    this.audio.pause()
    this.audio.removeAttribute("src")

    this.youtubeActive = true
    this.currentIsLive = isLive || false
    this.currentTrackId = trackId
    this.titleTarget.textContent = title
    this.artistTarget.textContent = artist

    if (this.currentIsLive) {
      this.showLiveMode()
    } else {
      this.showNormalMode()
    }

    // Delegate to YouTube player
    document.dispatchEvent(new CustomEvent("youtube:play", {
      detail: { videoId: youtubeVideoId, isLive: this.currentIsLive }
    }))

    if ("mediaSession" in navigator) {
      navigator.mediaSession.metadata = new MediaMetadata({
        title,
        artist: isLive ? "Live" : artist
      })
    }

    // Don't record play history for live streams
    if (!isLive && trackId) {
      this.recordPlay(trackId)
    }
  }

  toggle() {
    if (this.youtubeActive) {
      document.dispatchEvent(new CustomEvent("youtube:toggle"))
    } else {
      if (this.audio.paused) {
        this.audio.play()
      } else {
        this.audio.pause()
      }
    }
  }

  previous() {
    document.dispatchEvent(new CustomEvent("queue:previous"))
  }

  next() {
    document.dispatchEvent(new CustomEvent("queue:next"))
  }

  seek(event) {
    if (this.youtubeActive) return // YouTube seeking not supported via progress bar
    const rect = event.currentTarget.getBoundingClientRect()
    const percent = (event.clientX - rect.left) / rect.width
    this.audio.currentTime = percent * this.audio.duration
  }

  setVolume() {
    this.audio.volume = this.volumeTarget.value
  }

  // Local audio events

  onTimeUpdate() {
    if (this.youtubeActive) return
    if (this.audio.duration) {
      const percent = (this.audio.currentTime / this.audio.duration) * 100
      this.progressTarget.style.width = `${percent}%`
      this.currentTimeTarget.textContent = this.formatTime(this.audio.currentTime)
    }
  }

  onLoadedMetadata() {
    if (this.youtubeActive) return
    this.durationTarget.textContent = this.formatTime(this.audio.duration)
  }

  onEnded() {
    document.dispatchEvent(new CustomEvent("queue:next"))
  }

  // YouTube events

  onYouTubeState({ state }) {
    if (!this.youtubeActive) return

    if (state === "playing") {
      this.youtubePlaying = true
      this.playIconTarget.classList.add("hidden")
      this.pauseIconTarget.classList.remove("hidden")
    } else if (state === "paused") {
      this.youtubePlaying = false
      this.playIconTarget.classList.remove("hidden")
      this.pauseIconTarget.classList.add("hidden")
    } else if (state === "ended") {
      this.youtubePlaying = false
      document.dispatchEvent(new CustomEvent("queue:next"))
    }
  }

  onYouTubeTime({ currentTime, duration }) {
    if (!this.youtubeActive || this.currentIsLive) return

    if (duration > 0) {
      const percent = (currentTime / duration) * 100
      this.progressTarget.style.width = `${percent}%`
      this.currentTimeTarget.textContent = this.formatTime(currentTime)
      this.durationTarget.textContent = this.formatTime(duration)
    }
  }

  onYouTubeStopped() {
    this.youtubeActive = false
    this.youtubePlaying = false
    this.showNormalMode()
    this.updatePlayPauseIcon()
  }

  // UI modes

  showLiveMode() {
    if (this.hasProgressBarTarget) {
      this.progressBarTarget.classList.add("hidden")
    }
    if (this.hasLiveIndicatorTarget) {
      this.liveIndicatorTarget.classList.remove("hidden")
    }
    this.currentTimeTarget.textContent = ""
    this.durationTarget.textContent = ""
    this.progressTarget.style.width = "0%"
  }

  showNormalMode() {
    if (this.hasProgressBarTarget) {
      this.progressBarTarget.classList.remove("hidden")
    }
    if (this.hasLiveIndicatorTarget) {
      this.liveIndicatorTarget.classList.add("hidden")
    }
  }

  updatePlayPauseIcon() {
    if (this.youtubeActive) return // YouTube state change handles this

    if (this.audio.paused) {
      this.playIconTarget.classList.remove("hidden")
      this.pauseIconTarget.classList.add("hidden")
    } else {
      this.playIconTarget.classList.add("hidden")
      this.pauseIconTarget.classList.remove("hidden")
    }
  }

  recordPlay(trackId) {
    if (this.playHistoryUrlValue && trackId) {
      fetch(this.playHistoryUrlValue, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "X-CSRF-Token": document.querySelector('meta[name="csrf-token"]').content
        },
        body: JSON.stringify({ track_id: trackId })
      })
    }
  }

  formatTime(seconds) {
    if (!seconds || isNaN(seconds)) return "0:00"
    const mins = Math.floor(seconds / 60)
    const secs = Math.floor(seconds % 60)
    return `${mins}:${secs.toString().padStart(2, "0")}`
  }
}
