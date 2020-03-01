import { TurboHeartsService } from "../game/TurboHeartsService";

export class ChatInput {
  constructor(private input: HTMLTextAreaElement, private service: TurboHeartsService) {
    input.addEventListener("keypress", this.onKeyPress);
  }

  private onKeyPress = (event: KeyboardEvent) => {
    if (event.keyCode === 13) {
      this.service.chat(this.input.value);
      this.input.value = "";
      event.preventDefault();
      return false;
    }
  };
}
