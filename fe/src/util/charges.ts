import { Card, WithCharged } from "../types";

export function isCharged(c: Card, charges: WithCharged) {
  return (
    (c === "AH" && charges.chargedAh) ||
    (c === "JD" && charges.chargedJd) ||
    (c === "TC" && charges.chargedTc) ||
    (c === "QS" && charges.chargedQs)
  );
}
