import {
  Component,
  Inject,
  signal,
  computed,
  HostListener,
  ChangeDetectionStrategy,
} from "@angular/core";
import { FormsModule } from "@angular/forms";
import { MAT_DIALOG_DATA, MatDialogRef } from "@angular/material/dialog";
import { MatInput } from "@angular/material/input";
import { MatFormFieldModule } from "@angular/material/form-field";
import { MatButtonModule } from "@angular/material/button";
import { MatDialogModule } from "@angular/material/dialog";

export type NameInputDialogData = {
  label: string;
  placeholder: string;
};

@Component({
  selector: "name-input-dialog",
  templateUrl: "./name-input-dialog.component.html",
  styleUrl: "./name-input-dialog.component.scss",
  imports: [
    FormsModule,
    MatInput,
    MatFormFieldModule,
    MatButtonModule,
    MatDialogModule,
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class NameInputDialogComponent {
  public value = signal("");
  public isInvalid = computed(() => this.value() === "");

  constructor(
    @Inject(MAT_DIALOG_DATA) public data: NameInputDialogData,
    private readonly dialogRef: MatDialogRef<NameInputDialogComponent>,
  ) {}

  @HostListener("window:keyup.enter")
  public onEnter() {
    this.accept();
  }

  public accept() {
    this.dialogRef.close(this.value());
  }
}
