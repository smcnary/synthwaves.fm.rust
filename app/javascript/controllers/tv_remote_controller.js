import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  static values = { channels: Array, autoplayUrl: String, autoplayName: String }

  connect() {
    const player = document.getElementById("tv-player")
    if (!player) return

    if (this.hasChannelsValue && this.channelsValue.length > 0) {
      const retroTv = this.application.getControllerForElementAndIdentifier(player, "retro-tv")
      if (retroTv) retroTv.channelsValue = this.channelsValue
    }

    // Auto-play for channel show page
    if (this.hasAutoplayUrlValue && this.autoplayUrlValue) {
      const videoEvent = this.dispatchVideoEvent(this.autoplayUrlValue, this.autoplayNameValue)
      if (videoEvent.defaultPrevented) return

      const hlsCtrl = this.application.getControllerForElementAndIdentifier(player, "hls-player")
      if (hlsCtrl) hlsCtrl.play({ detail: { url: this.autoplayUrlValue, name: this.autoplayNameValue } })
    }
  }

  tune(event) {
    const { url, name, index } = event.params
    const videoEvent = this.dispatchVideoEvent(url, name)
    if (videoEvent.defaultPrevented) return

    const player = document.getElementById("tv-player")
    if (!player) return
    const hlsCtrl = this.application.getControllerForElementAndIdentifier(player, "hls-player")
    if (hlsCtrl) hlsCtrl.play({ detail: { url, name } })
    const retroCtrl = this.application.getControllerForElementAndIdentifier(player, "retro-tv")
    if (retroCtrl) retroCtrl.sync({ params: { index: parseInt(index, 10) } })
  }

  dispatchVideoEvent(url, name) {
    const event = new CustomEvent("video:playNow", {
      detail: { url, name, type: "hls_channel" },
      cancelable: true
    })
    document.dispatchEvent(event)
    return event
  }
}
