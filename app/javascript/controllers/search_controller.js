import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  static targets = ["form", "input"]

  connect() {
    this.timeout = null

    if (this.hasInputTarget && this.inputTarget.value) {
      this.inputTarget.focus()
      const len = this.inputTarget.value.length
      this.inputTarget.setSelectionRange(len, len)
    }
  }

  submit() {
    clearTimeout(this.timeout)
    this.timeout = setTimeout(() => {
      this.formTarget.requestSubmit()
    }, 300)
  }
}
