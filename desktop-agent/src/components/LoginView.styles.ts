import { CSSProperties } from 'react';

// 스타일 객체 타입 정의 (자동 완성을 위해)
type Styles = {
  [key: string]: CSSProperties;
};

export const styles: Styles = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    height: '100vh', 
    width: '100vw',  
    minWidth: '350px',
    minHeight: '500px',
    backgroundColor: '#111827', // gray-900
    color: 'white',
    padding: '24px',
    fontFamily: 'sans-serif',
    boxSizing: 'border-box',
    overflow: 'hidden',
  },
  header: {
    marginBottom: '40px',
    textAlign: 'center',
  },
  title: {
    fontSize: '36px',
    fontWeight: '800',
    color: '#4ade80', // green-400
    letterSpacing: '0.05em',
    marginBottom: '8px',
    marginTop: 0,
  },
  subtitle: {
    color: '#9ca3af', // gray-400
    fontSize: '14px',
    margin: 0,
  },
  buttonContainer: {
    width: '100%',
    maxWidth: '320px',
    display: 'flex',
    flexDirection: 'column',
    gap: '16px',
  },
  googleButton: {
    width: '100%',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    padding: '12px 16px',
    backgroundColor: 'white',
    color: '#1f2937', // gray-800
    fontWeight: 'bold',
    fontSize: '14px',
    borderRadius: '12px',
    border: 'none',
    boxShadow: '0 10px 15px -3px rgba(0, 0, 0, 0.1)',
    cursor: 'pointer',
    transition: 'all 0.2s',
    outline: 'none',
  },
  googleIcon: {
    width: '20px',
    height: '20px',
    marginRight: '12px',
  },
  dividerContainer: {
    display: 'flex',
    alignItems: 'center',
    padding: '4px 0',
    width: '100%',
  },
  dividerLine: {
    flexGrow: 1,
    height: '1px',
    backgroundColor: '#4b5563', // gray-600
  },
  dividerText: {
    flexShrink: 0,
    margin: '0 16px',
    color: '#6b7280', // gray-500
    fontSize: '12px',
    textTransform: 'uppercase',
    fontWeight: '600',
  },
  offlineButton: {
    width: '100%',
    padding: '12px 16px',
    backgroundColor: 'rgba(55, 65, 81, 0.5)', // gray-700/50
    color: '#d1d5db', // gray-300
    fontWeight: '500',
    fontSize: '14px',
    borderRadius: '12px',
    border: '1px solid #4b5563', // gray-600
    cursor: 'pointer',
    transition: 'all 0.2s',
    outline: 'none',
  },
  footer: {
    marginTop: '48px',
    textAlign: 'center',
  },
  footerText: {
    fontSize: '12px',
    color: '#4b5563', // gray-600
    margin: 0,
  }
};