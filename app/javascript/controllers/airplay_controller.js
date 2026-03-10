import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  static targets = ["button"]

  connect() {
    // AirPlay is Safari-only
    if (!window.WebKitPlaybackTargetAvailabilityEvent) return

    this._availabilityHandler = (e) => {
      if (e.availability === "available") {
        this.buttonTarget.classList.remove("hidden")
      } else {
        this.buttonTarget.classList.add("hidden")
      }
    }

    this._bindToAudio()

    // The persistent-audio element is created lazily by the player controller,
    // so watch for it if it doesn't exist yet.
    if (!this._audio()) {
      this._observer = new MutationObserver(() => {
        if (this._audio()) {
          this._bindToAudio()
          this._observer.disconnect()
          this._observer = null
        }
      })
      this._observer.observe(document.documentElement, { childList: true })
    }
  }

  disconnect() {
    const audio = this._audio()
    if (audio && this._availabilityHandler) {
      audio.removeEventListener("webkitplaybacktargetavailabilitychanged", this._availabilityHandler)
    }
    if (this._observer) {
      this._observer.disconnect()
      this._observer = null
    }
  }

  pick() {
    const audio = this._audio()
    if (audio?.webkitShowPlaybackTargetPicker) {
      audio.webkitShowPlaybackTargetPicker()
    }
  }

  _bindToAudio() {
    const audio = this._audio()
    if (audio && this._availabilityHandler) {
      audio.addEventListener("webkitplaybacktargetavailabilitychanged", this._availabilityHandler)
    }
  }

  _audio() {
    return document.getElementById("persistent-audio")
  }
}
