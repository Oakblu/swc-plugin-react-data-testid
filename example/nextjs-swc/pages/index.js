import React, { useState } from "react";
import { UserCard, ProductCard } from "../src/components/Cards";
import { Button, IconButton } from "../src/components/Buttons";
import Modal from "../src/components/Modal";
import ClassComponent from "../src/components/ClassComponent";
import SimpleTest from "../src/SimpleTest";
import EdgeCasesShowcase from "../src/components/EdgeCasesShowcase";

export default function Home() {
  const [showModal, setShowModal] = useState(false);
  const [counter, setCounter] = useState(0);

  return (
    <div className="app">
      <div className="container">
        <header className="hero">
          <div className="hero-content">
            <h1 className="hero-title">
              ⚡ <span className="gradient-text">Data TestId Generator</span>
            </h1>
            <p className="hero-subtitle">
              Automatic test ID generation for React components using Next.js + SWC plugin
            </p>

            <div className="highlight-card">
              <div className="highlight-icon">🔍</div>
              <div>
                <strong>Check the DOM inspector!</strong>
                <br />
                All elements have automatically generated <code>data-testid</code> attribute.
              </div>
            </div>
          </div>
        </header>

        <main className="main-content">
          <section className="demo-section">
            <div className="section-header">
              <h2>Interactive Components</h2>
              <p>Test the counter and modal functionality</p>
            </div>
            <div className="button-group">
              <Button onClick={() => setCounter(counter + 1)}>
                🔢 Counter: {counter}
              </Button>
              <IconButton icon="❤️" onClick={() => alert("Liked!")}>
                Like
              </IconButton>
              <Button onClick={() => setShowModal(true)}>
                🎭 Open Modal
              </Button>
            </div>
          </section>

          <section className="demo-section">
            <div className="section-header">
              <h2>Card Components</h2>
              <p>Beautiful card layouts with generated test IDs</p>
            </div>
            <div className="cards-grid">
              <UserCard name="John Doe" email="john@example.com" avatar="👨‍💻" />
              <ProductCard
                title="Amazing Widget"
                price="$99.99"
                description="This widget will change your life!"
              />
            </div>
          </section>

          <section className="demo-section">
            <div className="section-header">
              <h2>Plugin Test</h2>
              <p>Debug and testing components</p>
            </div>
            <div className="test-components">
              <SimpleTest />
              <ClassComponent />
            </div>
          </section>

          <section className="demo-section">
            <div className="section-header">
              <h2>Dynamic Content</h2>
              <p>Inline JSX and conditional rendering</p>
            </div>

            <div className="content-grid">
              <div className="content-card">
                <h3>Static List</h3>
                <span className="description">Basic JSX elements</span>
                <ul className="styled-list">
                  <li>First item</li>
                  <li>Second item</li>
                  <li>Third item</li>
                </ul>
              </div>

              <div className="content-card">
                <h3>Conditional Content</h3>
                <span className="description">Rendered when counter greater than 5</span>
                {counter > 5 ? (
                  <div className="conditional-content">
                    <p className="success-message">🎉 Counter is greater than 5!</p>
                    <button className="reset-btn" onClick={() => setCounter(0)}>
                      🔄 Reset Counter
                    </button>
                  </div>
                ) : (
                  <p className="info-message">Click the counter button above to see conditional content</p>
                )}
              </div>
            </div>
          </section>

          <section className="demo-section">
            <div className="section-header">
              <h2>Edge Case Behaviors</h2>
              <p>Open DevTools and inspect each element to verify the generated testids match the labels</p>
            </div>
            <EdgeCasesShowcase />
          </section>
        </main>

        {showModal && (
          <Modal onClose={() => setShowModal(false)}>
            <h3>⚡ SWC Modal</h3>
            <p>This modal also has auto-generated test IDs!</p>
            <div className="modal-actions">
              <button className="modal-btn primary" onClick={() => setShowModal(false)}>
                ✅ Close Modal
              </button>
            </div>
          </Modal>
        )}

        <footer className="footer">
          <div className="footer-content">
            <p>🎯 <strong>All elements have unique data-testid attributes</strong> generated automatically!</p>
            <small>Powered by swc-plugin-react-data-testid + Next.js ⚡</small>
          </div>
        </footer>
      </div>

      <style jsx>{`
        .app {
          min-height: 100vh;
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
        }

        .container {
          max-width: 1000px;
          margin: 0 auto;
          padding: 20px;
        }

        .hero {
          text-align: center;
          padding: 60px 0;
          margin-bottom: 40px;
        }

        .hero-content {
          background: rgba(255, 255, 255, 0.95);
          backdrop-filter: blur(10px);
          border-radius: 20px;
          padding: 40px;
          box-shadow: 0 20px 40px rgba(0, 0, 0, 0.1);
        }

        .hero-title {
          font-size: 3rem;
          margin: 0 0 20px 0;
          font-weight: 700;
        }

        .gradient-text {
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          -webkit-background-clip: text;
          -webkit-text-fill-color: transparent;
          background-clip: text;
        }

        .hero-subtitle {
          font-size: 1.2rem;
          color: #666;
          margin-bottom: 30px;
          line-height: 1.6;
        }

        .highlight-card {
          background: linear-gradient(135deg, #ffeaa7 0%, #fab1a0 100%);
          padding: 20px;
          border-radius: 15px;
          display: flex;
          align-items: center;
          gap: 15px;
          box-shadow: 0 10px 20px rgba(0, 0, 0, 0.1);
          max-width: 600px;
          margin: 0 auto;
        }

        .highlight-icon {
          font-size: 2rem;
        }

        .main-content {
          display: flex;
          flex-direction: column;
          gap: 40px;
        }

        .demo-section {
          background: rgba(255, 255, 255, 0.95);
          border-radius: 20px;
          padding: 30px;
          box-shadow: 0 10px 30px rgba(0, 0, 0, 0.1);
          backdrop-filter: blur(10px);
        }

        .section-header {
          text-align: center;
          margin-bottom: 30px;
        }

        .section-header h2 {
          font-size: 2rem;
          margin: 0 0 10px 0;
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          -webkit-background-clip: text;
          -webkit-text-fill-color: transparent;
          background-clip: text;
        }

        .section-header p {
          color: #666;
          font-size: 1.1rem;
          margin: 0;
        }

        .button-group {
          display: flex;
          justify-content: center;
          gap: 15px;
          flex-wrap: wrap;
        }

        .cards-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
          gap: 20px;
        }

        .test-components {
          display: flex;
          gap: 20px;
          justify-content: center;
          flex-wrap: wrap;
        }

        .content-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
          gap: 20px;
        }

        .content-card {
          background: #f8f9fa;
          border-radius: 15px;
          padding: 20px;
          border: 2px solid #e9ecef;
          transition: all 0.3s ease;
        }

        .content-card:hover {
          transform: translateY(-5px);
          box-shadow: 0 15px 30px rgba(0, 0, 0, 0.1);
        }

        .content-card h3 {
          margin: 0 0 5px 0;
          color: #495057;
        }

        .description {
          color: #6c757d;
          font-size: 0.9rem;
          margin-bottom: 15px;
          display: block;
        }

        .styled-list {
          list-style: none;
          padding: 0;
        }

        .styled-list li {
          background: linear-gradient(135deg, #84fab0 0%, #8fd3f4 100%);
          margin: 8px 0;
          padding: 10px 15px;
          border-radius: 8px;
          color: #333;
          font-weight: 500;
        }

        .conditional-content {
          text-align: center;
        }

        .success-message {
          color: #28a745;
          font-weight: 600;
          margin-bottom: 15px;
        }

        .info-message {
          color: #6c757d;
          font-style: italic;
        }

        .reset-btn,
        .modal-btn {
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          color: white;
          border: none;
          padding: 12px 24px;
          border-radius: 8px;
          cursor: pointer;
          font-weight: 600;
          transition: all 0.3s ease;
        }

        .reset-btn:hover,
        .modal-btn:hover {
          transform: translateY(-2px);
          box-shadow: 0 10px 20px rgba(0, 0, 0, 0.2);
        }

        .modal-actions {
          text-align: center;
          margin-top: 20px;
        }

        .footer {
          margin-top: 60px;
          text-align: center;
        }

        .footer-content {
          background: rgba(255, 255, 255, 0.95);
          backdrop-filter: blur(10px);
          border-radius: 15px;
          padding: 30px;
          box-shadow: 0 10px 30px rgba(0, 0, 0, 0.1);
        }

        .footer-content p {
          margin: 0 0 10px 0;
          font-size: 1.1rem;
          color: #333;
        }

        .footer-content small {
          color: #666;
          font-size: 0.9rem;
        }

        code {
          background: rgba(233, 236, 239, 0.8);
          padding: 3px 6px;
          border-radius: 4px;
          font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Fira Code', monospace;
          font-size: 0.9em;
          color: #495057;
        }

        @media (max-width: 768px) {
          .hero-title {
            font-size: 2rem;
          }

          .container {
            padding: 15px;
          }

          .hero {
            padding: 40px 0;
          }

          .hero-content {
            padding: 30px 20px;
          }

          .highlight-card {
            flex-direction: column;
            text-align: center;
          }
        }
      `}</style>
    </div>
  );
}
