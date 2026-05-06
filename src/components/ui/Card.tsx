import type { ReactNode } from "react";

type CardProps = {
  title?: string;
  description?: string;
  eyebrow?: string;
  actions?: ReactNode;
  className?: string;
  children: ReactNode;
};

function Card({
  title,
  description,
  eyebrow,
  actions,
  className,
  children,
}: CardProps) {
  const classes = ["card"];
  if (className) {
    classes.push(className);
  }

  return (
    <section className={classes.join(" ")}>
      {title || description || eyebrow || actions ? (
        <header className="card-header">
          <div>
            {eyebrow ? <p className="card-eyebrow">{eyebrow}</p> : null}
            {title ? <h2 className="card-title">{title}</h2> : null}
            {description ? <p className="card-description">{description}</p> : null}
          </div>
          {actions ? <div className="card-actions">{actions}</div> : null}
        </header>
      ) : null}
      {children}
    </section>
  );
}

export default Card;
