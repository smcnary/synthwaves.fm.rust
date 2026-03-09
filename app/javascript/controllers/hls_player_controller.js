import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  static targets = ["video", "container", "placeholder", "error", "loading", "channelName", "ccButton", "videoWrapper"]
  static values = { autoplayUrl: String, autoplayName: String }

  connect() {
    // Guard: skip re-init if already connected (data-turbo-permanent)
    if (this.element._hlsPlayerConnected) return
    this.element._hlsPlayerConnected = true

    this.hls = null
    this.networkRetries = 0
    this.currentChannelName = null
    this.subtitlesEnabled = localStorage.getItem("hlsPlayerCC") === "on"
    this._onTvPage = false

    // Pre-load hls.js so it's available synchronously on click
    this.hlsClass = null
    import("hls.js").then(mod => { this.hlsClass = mod.default }).catch(() => {})

    // Listen for page lifecycle events
    this._onEnterTv = () => this.enterTvPage()
    this._onLeaveTv = () => this.leaveTvPage()
    this._onPipClose = () => this.close()
    this._onPipOpen = () => this.openFromPip()

    document.addEventListener("tv:enterTvPage", this._onEnterTv)
    document.addEventListener("tv:leaveTvPage", this._onLeaveTv)
    document.addEventListener("pip:close", this._onPipClose)
    document.addEventListener("pip:open", this._onPipOpen)

    // Auto-play on show page (works with Turbo navigation)
    if (this.hasAutoplayUrlValue && this.autoplayUrlValue) {
      this.play({ detail: { url: this.autoplayUrlValue, name: this.autoplayNameValue } })
    }
  }

  disconnect() {
    // With data-turbo-permanent, don't destroy anything.
    // Only clean up event listeners if the element is truly removed.
    if (!document.contains(this.element)) {
      this.element._hlsPlayerConnected = false
      document.removeEventListener("tv:enterTvPage", this._onEnterTv)
      document.removeEventListener("tv:leaveTvPage", this._onLeaveTv)
      document.removeEventListener("pip:close", this._onPipClose)
      document.removeEventListener("pip:open", this._onPipOpen)
      this.destroyPlayer()
    }
  }

  play(event) {
    const { url, name } = event.params || event.detail || {}
    if (!url) return

    this.networkRetries = 0
    this.currentChannelName = name
    this.showLoading()
    this.hideError()
    this.updateChannelName(name)

    // Pause any music/YouTube playing through the bottom player
    const audio = document.getElementById("persistent-audio")
    if (audio && !audio.paused) audio.pause()
    document.dispatchEvent(new CustomEvent("youtube:stop"))

    // Show expanded view if on TV page, otherwise show PiP
    if (this.isOnTvPage()) {
      this.showExpanded()
    } else {
      this.showPip(name)
    }

    const video = this.videoTarget
    video.classList.remove("hidden")
    if (this.hasPlaceholderTarget) this.placeholderTarget.classList.add("hidden")

    this.destroyPlayer()

    if (video.canPlayType("application/vnd.apple.mpegurl")) {
      this.playNative(video, url)
    } else if (this.hlsClass) {
      this.setupHls(video, url)
    } else {
      this.loadHlsAsync(video, url)
    }
  }

  close() {
    this.destroyPlayer()
    this.videoTarget.pause()
    this.videoTarget.removeAttribute("src")
    this.videoTarget.load()
    this.currentChannelName = null

    // Hide both views
    if (this.hasContainerTarget) this.containerTarget.classList.add("hidden")
    this.hidePip()

    // Ensure video is back in expanded wrapper
    this.moveVideoToExpanded()

    // Show music player bar
    this.showPlayerBar()
  }

  // --- Expanded / PiP transitions ---

  enterTvPage() {
    this._onTvPage = true
    this.hidePlayerBar()
    this.pauseAllAudio()

    // If playing, move from PiP to expanded
    if (this.isPlaying()) {
      this.showExpanded()
      this.moveVideoToExpanded()
      this.hidePip()
    }
  }

  leaveTvPage() {
    this._onTvPage = false
    this.showPlayerBar()

    // If playing, minimize to PiP
    if (this.isPlaying()) {
      this.hideExpanded()
      this.moveVideoToPip()
      this.showPip(this.currentChannelName)
    }
  }

  openFromPip() {
    // Navigate back to TV page
    if (typeof Turbo !== "undefined") {
      Turbo.visit("/tv")
    } else {
      window.location.href = "/tv"
    }
  }

  showExpanded() {
    if (this.hasContainerTarget) this.containerTarget.classList.remove("hidden")
  }

  hideExpanded() {
    if (this.hasContainerTarget) this.containerTarget.classList.add("hidden")
  }

  moveVideoToExpanded() {
    if (this.hasVideoWrapperTarget) {
      const video = this.videoTarget
      if (video.parentElement !== this.videoWrapperTarget) {
        this.videoWrapperTarget.insertBefore(video, this.videoWrapperTarget.firstChild)
      }
    }
  }

  moveVideoToPip() {
    const pip = document.getElementById("pip-player")
    if (!pip) return
    const slot = pip.querySelector("[data-pip-target='slot']")
    if (!slot) return
    slot.appendChild(this.videoTarget)
  }

  showPip(title) {
    const pip = document.getElementById("pip-player")
    if (!pip) return
    const pipCtrl = this.application.getControllerForElementAndIdentifier(pip, "pip")
    if (pipCtrl) pipCtrl.show(title)
  }

  hidePip() {
    const pip = document.getElementById("pip-player")
    if (!pip) return
    const pipCtrl = this.application.getControllerForElementAndIdentifier(pip, "pip")
    if (pipCtrl) pipCtrl.hide()
  }

  isPlaying() {
    return this.videoTarget && !this.videoTarget.paused
  }

  isOnTvPage() {
    return this._onTvPage || !!document.querySelector("[data-controller~='tv-page']")
  }

  // --- Player bar visibility ---

  hidePlayerBar() {
    const playerBar = document.getElementById("player-bar")
    if (playerBar) playerBar.classList.add("hidden")
    const queuePanel = document.getElementById("queue-panel-container")
    if (queuePanel) queuePanel.classList.add("hidden")
    document.body.classList.remove("pb-24")
  }

  showPlayerBar() {
    const playerBar = document.getElementById("player-bar")
    if (playerBar) playerBar.classList.remove("hidden")
    const queuePanel = document.getElementById("queue-panel-container")
    if (queuePanel) queuePanel.classList.remove("hidden")
    document.body.classList.add("pb-24")
  }

  pauseAllAudio() {
    const audio = document.getElementById("persistent-audio")
    if (audio && !audio.paused) audio.pause()
    document.dispatchEvent(new CustomEvent("youtube:stop"))
  }

  // --- HLS playback ---

  playNative(video, url) {
    video.src = url

    video.onerror = () => {
      this.showError("Stream unavailable. The channel may be offline.")
    }

    video.textTracks.addEventListener("addtrack", () => this.detectSubtitles())
    video.textTracks.addEventListener("change", () => this.syncCCButtonState())

    video.play().then(() => {
      this.hideLoading()
    }).catch(() => {
      this.hideLoading()
    })
  }

  setupHls(video, url) {
    const Hls = this.hlsClass
    if (!Hls.isSupported()) {
      this.showError("HLS playback is not supported in this browser.")
      return
    }

    this.hls = new Hls({
      enableWorker: true,
      lowLatencyMode: true,
      enableCEA708Captions: true,
      renderTextTracksNatively: true
    })

    this.hls.on(Hls.Events.MANIFEST_PARSED, () => {
      setTimeout(() => this.detectSubtitles(), 1500)
      video.play().then(() => {
        this.hideLoading()
      }).catch(() => {
        this.hideLoading()
      })
    })

    this.hls.on(Hls.Events.SUBTITLE_TRACKS_UPDATED, () => this.detectSubtitles())

    this.hls.on(Hls.Events.ERROR, (_event, data) => {
      if (data.fatal) this.handleHlsError(Hls, data)
    })

    video.textTracks.addEventListener("addtrack", () => this.detectSubtitles())

    this.hls.loadSource(url)
    this.hls.attachMedia(video)
  }

  async loadHlsAsync(video, url) {
    try {
      const { default: Hls } = await import("hls.js")
      this.hlsClass = Hls
      this.setupHls(video, url)
    } catch {
      this.showError("Failed to load HLS player.")
    }
  }

  handleHlsError(Hls, data) {
    switch (data.type) {
      case Hls.ErrorTypes.NETWORK_ERROR:
        this.networkRetries++
        if (this.networkRetries <= 3) {
          this.hls.startLoad()
        } else {
          this.showError("Stream unavailable. It may be offline or blocked by CORS.")
          this.destroyPlayer()
        }
        break
      case Hls.ErrorTypes.MEDIA_ERROR:
        this.hls.recoverMediaError()
        break
      default:
        this.showError("Stream unavailable. The channel may be offline.")
        this.destroyPlayer()
        break
    }
  }

  destroyPlayer() {
    if (this.hls) {
      this.hls.destroy()
      this.hls = null
    }
    this.hideCCButton()
  }

  // --- UI helpers ---

  showLoading() {
    if (this.hasLoadingTarget) this.loadingTarget.classList.remove("hidden")
  }

  hideLoading() {
    if (this.hasLoadingTarget) this.loadingTarget.classList.add("hidden")
  }

  showError(message) {
    this.hideLoading()
    if (this.hasErrorTarget) {
      this.errorTarget.textContent = message
      this.errorTarget.classList.remove("hidden")
    }
  }

  hideError() {
    if (this.hasErrorTarget) this.errorTarget.classList.add("hidden")
  }

  updateChannelName(name) {
    if (this.hasChannelNameTarget && name) {
      this.channelNameTarget.textContent = name
    }
  }

  // --- Closed Caption / Subtitle support ---

  detectSubtitles() {
    const video = this.videoTarget
    const hasHlsSubtitles = this.hls?.subtitleTracks?.length > 0
    const hasNativeTracks = this.hasSubtitleTextTracks(video)

    if (hasHlsSubtitles || hasNativeTracks) {
      this.showCCButton()
      this.applySubtitlePreference(video)
    }
  }

  hasSubtitleTextTracks(video) {
    for (let i = 0; i < video.textTracks.length; i++) {
      const track = video.textTracks[i]
      if (track.kind === "subtitles" || track.kind === "captions") return true
    }
    return false
  }

  applySubtitlePreference(video) {
    const mode = this.subtitlesEnabled ? "showing" : "hidden"
    for (let i = 0; i < video.textTracks.length; i++) {
      const track = video.textTracks[i]
      if (track.kind === "subtitles" || track.kind === "captions") {
        track.mode = mode
      }
    }
    this.updateCCButtonState()
  }

  toggleCC() {
    this.subtitlesEnabled = !this.subtitlesEnabled
    localStorage.setItem("hlsPlayerCC", this.subtitlesEnabled ? "on" : "off")
    this.applySubtitlePreference(this.videoTarget)
  }

  syncCCButtonState() {
    const video = this.videoTarget
    let anyShowing = false
    for (let i = 0; i < video.textTracks.length; i++) {
      const track = video.textTracks[i]
      if ((track.kind === "subtitles" || track.kind === "captions") && track.mode === "showing") {
        anyShowing = true
        break
      }
    }
    if (anyShowing !== this.subtitlesEnabled) {
      this.subtitlesEnabled = anyShowing
      localStorage.setItem("hlsPlayerCC", this.subtitlesEnabled ? "on" : "off")
      this.updateCCButtonState()
    }
  }

  showCCButton() {
    if (this.hasCcButtonTarget) this.ccButtonTarget.classList.remove("hidden")
    this.updateCCButtonState()
  }

  hideCCButton() {
    if (this.hasCcButtonTarget) this.ccButtonTarget.classList.add("hidden")
  }

  updateCCButtonState() {
    if (!this.hasCcButtonTarget) return
    const btn = this.ccButtonTarget
    if (this.subtitlesEnabled) {
      btn.classList.remove("border-gray-500", "text-gray-400")
      btn.classList.add("border-neon-cyan", "text-neon-cyan")
      btn.style.textShadow = "0 0 6px rgba(0, 255, 255, 0.6)"
    } else {
      btn.classList.remove("border-neon-cyan", "text-neon-cyan")
      btn.classList.add("border-gray-500", "text-gray-400")
      btn.style.textShadow = ""
    }
  }
}
