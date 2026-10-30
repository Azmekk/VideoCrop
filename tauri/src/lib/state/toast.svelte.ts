export interface Toast { id: number; text: string }

class Toasts {
  items = $state<Toast[]>([]);
  #id = 0;

  show(text: string, ms = 3200) {
    const id = ++this.#id;
    this.items = [...this.items.filter((t) => t.text !== text), { id, text }];
    setTimeout(() => this.dismiss(id), ms);
  }

  dismiss(id: number) {
    this.items = this.items.filter((t) => t.id !== id);
  }
}

export const toasts = new Toasts();
