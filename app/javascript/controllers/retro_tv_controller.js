import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  static values = { channels: Array }
  static targets = ["nowPlaying", "channelNumber", "channelName", "channelLogo", "progressBar", "timeLeft"]

  connect() {
    // Guard: skip re-init if already connected (data-turbo-permanent)
    if (this.element._retroTvConnected) return
    this.element._retroTvConnected = true

    this.currentIndex = -1
    this.refreshTimer = setInterval(() => this.updateNowPlaying(), 60000)
  }

  disconnect() {
    // With data-turbo-permanent, don't clear the timer
    if (!document.contains(this.element)) {
      this.element._retroTvConnected = false
      if (this.refreshTimer) clearInterval(this.refreshTimer)
    }
  }

  channelUp() {
    if (this.channelsValue.length === 0) return
    this.currentIndex = this.currentIndex >= this.channelsValue.length - 1 ? 0 : this.currentIndex + 1
    this.tuneToChannel()
  }

  channelDown() {
    if (this.channelsValue.length === 0) return
    this.currentIndex = this.currentIndex <= 0 ? this.channelsValue.length - 1 : this.currentIndex - 1
    this.tuneToChannel()
  }

  sync(event) {
    const index = typeof event.params?.index === "number" ? event.params.index : parseInt(event.params?.index, 10)
    if (!isNaN(index) && index >= 0 && index < this.channelsValue.length) {
      this.currentIndex = index
      this.updateNowPlaying()
      this.updateChannelDisplay()
    }
  }

  keydown(event) {
    if (event.key === "ArrowUp") {
      event.preventDefault()
      this.channelUp()
    } else if (event.key === "ArrowDown") {
      event.preventDefault()
      this.channelDown()
    }
  }

  tuneToChannel() {
    const channel = this.channelsValue[this.currentIndex]
    if (!channel) return

    const videoEvent = new CustomEvent("video:playNow", {
      detail: { url: channel.streamUrl, name: channel.name, type: "hls_channel" },
      cancelable: true
    })
    document.dispatchEvent(videoEvent)

    if (!videoEvent.defaultPrevented) {
      const hlsPlayer = this.application.getControllerForElementAndIdentifier(this.element, "hls-player")
      if (hlsPlayer) {
        hlsPlayer.play({ detail: { url: channel.streamUrl, name: channel.name } })
      }
    }

    this.updateChannelDisplay()
    this.updateNowPlaying()
  }

  updateChannelDisplay() {
    const channel = this.channelsValue[this.currentIndex]
    if (!channel) return

    if (this.hasChannelNumberTarget) {
      this.channelNumberTargets.forEach(el => {
        el.textContent = String(this.currentIndex + 1).padStart(2, "0")
      })
    }
    if (this.hasChannelNameTarget) {
      this.channelNameTarget.textContent = channel.name
    }
    if (this.hasChannelLogoTarget) {
      if (channel.logoUrl) {
        this.channelLogoTarget.innerHTML = `<img src="${this.escapeHtml(channel.logoUrl)}" alt="" class="w-full h-full object-contain">`
      } else {
        const initials = channel.name.match(/[A-Z0-9]/g)?.slice(0, 3).join("") || channel.name.slice(0, 2).toUpperCase()
        this.channelLogoTarget.innerHTML = `<span class="text-[10px] font-bold text-white">${this.escapeHtml(initials)}</span>`
      }
    }
  }

  updateNowPlaying() {
    const channel = this.channelsValue[this.currentIndex]
    if (!channel) return

    const now = Math.floor(Date.now() / 1000)
    const current = channel.programmes?.find(p => p.startsAt <= now && p.endsAt > now)

    if (this.hasNowPlayingTarget) {
      this.nowPlayingTarget.textContent = current ? current.title : "No schedule info"
    }

    if (this.hasProgressBarTarget && this.hasTimeLeftTarget) {
      if (current) {
        const elapsed = now - current.startsAt
        const total = current.endsAt - current.startsAt
        const pct = Math.min(100, Math.round((elapsed / total) * 100))
        const minsLeft = Math.max(0, Math.ceil((current.endsAt - now) / 60))
        this.progressBarTarget.style.width = `${pct}%`
        this.progressBarTarget.parentElement.classList.remove("hidden")
        this.timeLeftTarget.textContent = `${minsLeft}min left`
        this.timeLeftTarget.classList.remove("hidden")
      } else {
        this.progressBarTarget.parentElement.classList.add("hidden")
        this.timeLeftTarget.classList.add("hidden")
      }
    }
  }

  escapeHtml(str) {
    const div = document.createElement("div")
    div.textContent = str
    return div.innerHTML
  }
}
