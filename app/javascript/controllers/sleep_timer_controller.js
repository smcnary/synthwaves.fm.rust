import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  static targets = ["badge", "menu"]

  static PRESETS = [15, 30, 60, 90, 120]

  connect() {
    this._checkExisting()
    this._tick = () => this._update()
    this._interval = setInterval(this._tick, 1000)
  }

  disconnect() {
    if (this._interval) clearInterval(this._interval)
  }

  start(event) {
    const minutes = parseInt(event.currentTarget.dataset.minutes)
    const endAt = Date.now() + minutes * 60 * 1000
    localStorage.setItem("sleepTimerEndAt", endAt.toString())
    this._endAt = endAt
    this._closeMenu()
    this._update()
  }

  cancel() {
    localStorage.removeItem("sleepTimerEndAt")
    this._endAt = null
    this._closeMenu()
    this._update()
  }

  toggleMenu() {
    if (this.hasMenuTarget) {
      this.menuTarget.classList.toggle("hidden")
    }
  }

  _closeMenu() {
    if (this.hasMenuTarget) {
      this.menuTarget.classList.add("hidden")
    }
  }

  _checkExisting() {
    const saved = localStorage.getItem("sleepTimerEndAt")
    if (saved) {
      const endAt = parseInt(saved)
      if (endAt > Date.now()) {
        this._endAt = endAt
      } else {
        localStorage.removeItem("sleepTimerEndAt")
      }
    }
  }

  _update() {
    if (!this._endAt) {
      this._showBadge(null)
      return
    }

    const remaining = this._endAt - Date.now()
    if (remaining <= 0) {
      this._expire()
      return
    }

    const mins = Math.floor(remaining / 60000)
    const secs = Math.floor((remaining % 60000) / 1000)
    this._showBadge(`${mins}:${secs.toString().padStart(2, "0")}`)
  }

  _expire() {
    localStorage.removeItem("sleepTimerEndAt")
    this._endAt = null
    this._showBadge(null)

    // Pause audio
    const audio = document.getElementById("persistent-audio")
    if (audio && !audio.paused) audio.pause()

    // Stop YouTube if playing
    document.dispatchEvent(new CustomEvent("youtube:stop"))
  }

  _showBadge(text) {
    if (!this.hasBadgeTarget) return
    if (text) {
      this.badgeTarget.textContent = text
      this.badgeTarget.classList.remove("hidden")
    } else {
      this.badgeTarget.classList.add("hidden")
    }
  }
}
