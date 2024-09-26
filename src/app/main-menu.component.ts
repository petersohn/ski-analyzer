import { Component, EventEmitter, Output } from "@angular/core";
import { MatButton } from "@angular/material/button";
import { open } from "@tauri-apps/plugin-dialog";

@Component({
  selector: "main-menu",
  standalone: true,
  imports: [MatButton],
  templateUrl: "./main-menu.component.html",
  styleUrls: ["./main-menu.component.css"],
})
export class MainMenuComponent {
  @Output()
  public onLoadSkiArea = new EventEmitter<string>();

  public async loadSkiArea() {
    const path = await open({
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (!!path) {
      this.onLoadSkiArea.emit(path);
    }
  }
}
