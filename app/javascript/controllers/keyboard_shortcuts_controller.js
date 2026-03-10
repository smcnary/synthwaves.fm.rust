import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  static targets = ["helpModal"]

  connect() {
    this._suppressed = false
    this._onKeydown = (e) => this._handleKeydown(e)
    this._onSuppress = () => { this._suppressed = true }
    this._onRestore = () => { this._suppressed = false }

    document.addEventListener("keydown", this._onKeydown)
    document.addEventListener("screensaver:activated", this._onSuppress)
    document.addEventListener("screensaver:dismissed", this._onRestore)
  }

  disconnect() {
    document.removeEventListener("keydown", this._onKeydown)
    document.removeEventListener("screensaver:activated", this._onSuppress)
    document.removeEventListener("screensaver:dismissed", this._onRestore)
  }

  _handleKeydown(event) {
    // Don't fire in inputs, textareas, contenteditable, or select elements
    const tag = event.target.tagName
    if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT" || event.target.isContentEditable) return

    // Don't fire when modifier keys are held (except Shift for ?)
    if (event.ctrlKey || event.metaKey || event.altKey) return

    if (this._suppressed) return

    const key = event.key

    switch (key) {
      case " ":
        event.preventDefault()
        this._dispatch("player", "toggle")
        break
      case "n":
      case "N":
        this._dispatch("queue", "next")
        break
      case "p":
      case "P":
        this._dispatch("queue", "previous")
        break
      case "m":
      case "M":
        this._dispatch("player", "toggleMute")
        break
      case "ArrowRight":
        event.preventDefault()
        this._dispatch("player", "seekForward")
        break
      case "ArrowLeft":
        event.preventDefault()
        this._dispatch("player", "seekBackward")
        break
      case "ArrowUp":
        event.preventDefault()
        this._dispatch("player", "volumeUp")
        break
      case "ArrowDown":
        event.preventDefault()
        this._dispatch("player", "volumeDown")
        break
      case "v":
      case "V":
        this._dispatch("visualizer-panel", "toggle")
        break
      case "q":
      case "Q":
        this._dispatch("queue-panel", "toggle")
        break
      case "s":
      case "S":
        this._dispatch("queue", "toggleShuffle")
        break
      case "r":
      case "R":
        this._dispatch("queue", "cycleRepeat")
        break
      case "f":
      case "F":
        this._dispatch("fullscreen-now-playing", "toggle")
        break
      case "Escape":
        this._dismissOverlays()
        break
      case "?":
        event.preventDefault()
        this._toggleHelp()
        break
      default:
        return
    }
  }

  _dispatch(namespace, action) {
    document.dispatchEvent(new CustomEvent(`${namespace}:${action}`))
  }

  _dismissOverlays() {
    // Close help modal
    if (this.hasHelpModalTarget && !this.helpModalTarget.classList.contains("hidden")) {
      this.helpModalTarget.classList.add("hidden")
      return
    }
    // Close queue panel
    this._dispatch("queue-panel", "close")
    // Close visualizer panel
    this._dispatch("visualizer-panel", "close")
    // Close fullscreen now playing
    this._dispatch("fullscreen-now-playing", "close")
  }

  _toggleHelp() {
    if (!this.hasHelpModalTarget) return
    this.helpModalTarget.classList.toggle("hidden")
  }

  closeHelp() {
    if (this.hasHelpModalTarget) {
      this.helpModalTarget.classList.add("hidden")
    }
  }
}
