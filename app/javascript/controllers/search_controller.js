import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  static targets = ["form", "input"]

  connect() {
    this.timeout = null
    this.abortController = null

    if (this.hasInputTarget && this.inputTarget.value) {
      this.inputTarget.focus()
      const len = this.inputTarget.value.length
      this.inputTarget.setSelectionRange(len, len)
    }

    this.boundAttachAbort = this.attachAbortSignal.bind(this)
    this.boundClearLoading = this.clearLoading.bind(this)

    this.formTarget.addEventListener("turbo:before-fetch-request", this.boundAttachAbort)
    this.formTarget.addEventListener("turbo:frame-load", this.boundClearLoading)
  }

  disconnect() {
    clearTimeout(this.timeout)
    this.abortController?.abort()
    this.formTarget.removeEventListener("turbo:before-fetch-request", this.boundAttachAbort)
    this.formTarget.removeEventListener("turbo:frame-load", this.boundClearLoading)
  }

  submit() {
    clearTimeout(this.timeout)
    this.timeout = setTimeout(() => {
      this.abortController?.abort()
      this.abortController = new AbortController()
      if (this.hasInputTarget) {
        this.inputTarget.style.opacity = "0.6"
      }
      this.formTarget.requestSubmit()
    }, 500)
  }

  attachAbortSignal(event) {
    if (this.abortController) {
      event.detail.fetchOptions.signal = this.abortController.signal
    }
  }

  clearLoading() {
    if (this.hasInputTarget) {
      this.inputTarget.style.opacity = "1"
    }
  }
}
