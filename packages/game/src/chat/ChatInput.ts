export class ChatInput {
  constructor(private input: HTMLTextAreaElement, private gameId: string) {
    input.addEventListener("keypress", this.onKeyPress);
  }

  private requestWithBody(body: any): RequestInit {
    return {
      credentials: "include",
      method: "POST",
      headers: {
        "Content-Type": "application/json"
      },
      body: JSON.stringify(body)
    };
  }

  public chat = (message: string) => {
    return fetch(`/game/chat`, this.requestWithBody({ id: this.gameId, message }));
  };

  private onKeyPress = (event: KeyboardEvent) => {
    if (event.keyCode === 13) {
      this.chat(this.input.value);
      this.input.value = "";
      event.preventDefault();
      return false;
    }
  };
}
