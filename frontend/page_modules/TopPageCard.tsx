import { Card } from "@mui/material";
import Link from "next/link";
import { ReactNode } from "react";

export const TopPageCard = (props: {
  cardHref: string;
  children: ReactNode;
}) => {
  return (
    <div style={{ width: "30rem", margin: "1rem" }}>
      <Card>
        <Link
          href={props.cardHref}
          style={{ textDecoration: "none", color: "black" }}
        >
          <div
            style={{
              minHeight: "3rem",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              padding: "1rem",
            }}
          >
            {props.children}
          </div>
        </Link>
      </Card>
    </div>
  );
};
