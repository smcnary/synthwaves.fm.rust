import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  static targets = ["slot", "title", "openButton"]

  show(title) {
    this.element.classList.remove("hidden")
    if (this.hasTitleTarget && title) {
      this.titleTarget.textContent = title
    }
  }

  hide() {
    this.element.classList.add("hidden")
  }

  close() {
    document.dispatchEvent(new CustomEvent("pip:close"))
    this.hide()
  }

  open() {
    document.dispatchEvent(new CustomEvent("pip:open"))
  }
}
