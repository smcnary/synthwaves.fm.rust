import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  static targets = ["input", "frame", "urlField"]
  static values = { url: String }

  submit(event) {
    event.preventDefault()
    const query = this.inputTarget.value.trim()
    if (query.length === 0) return

    this.frameTarget.src = `${this.urlValue}?q=${encodeURIComponent(query)}`
  }

  selectResult(event) {
    const url = event.params.url
    this.urlFieldTarget.value = url
    this.frameTarget.innerHTML = ""
    this.inputTarget.value = ""
  }
}
