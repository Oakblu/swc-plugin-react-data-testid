import React from "react";

export const UserCard = ({ name, email, avatar }) => {
  return (
    <div className="card">
      <div>
        <div>{avatar}</div>
        <div>
          <h3>{name}</h3>
          <p>{email}</p>
          <div>
            <button>Follow</button>
            <button>Message</button>
          </div>
        </div>
      </div>
    </div>
  );
};

export const ProductCard = ({ title, price, description }) => {
  return (
    <div className="card">
      <div>
        <img src="/api/placeholder/200/150" alt={title} />
      </div>
      <div>
        <h3>{title}</h3>
        <p>{description}</p>
        <div>
          <span>{price}</span>
          <button>Add to Cart</button>
        </div>
      </div>
    </div>
  );
};
