import { CSSProperties } from 'react';

type Styles = {
  [key: string]: CSSProperties;
};

export const styles: Styles = {
  container: {
    padding: '24px',
    fontFamily: 'sans-serif',
    color: 'white',
    height: '100%',
    boxSizing: 'border-box',
    display: 'flex',
    flexDirection: 'column',
    overflowY: 'auto', // 내용이 많을 경우 스크롤 허용
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '30px',
  },
  logo: {
    margin: 0,
    fontSize: '20px',
    fontWeight: '800',
    color: '#4ade80',
    letterSpacing: '0.05em',
  },
  statusContainer: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
  },
  statusBadge: {
    display: 'flex',
    alignItems: 'center',
    fontSize: '12px',
    color: '#d1d5db',
    backgroundColor: 'rgba(255, 255, 255, 0.05)',
    padding: '4px 8px',
    borderRadius: '12px',
  },
  statusDot: {
    width: '6px',
    height: '6px',
    borderRadius: '50%',
    marginRight: '6px',
  },
  logoutButton: {
    backgroundColor: 'transparent',
    border: '1px solid #4b5563',
    color: '#9ca3af',
    padding: '4px 10px',
    borderRadius: '6px',
    cursor: 'pointer',
    fontSize: '11px',
    transition: 'all 0.2s',
  },
  errorBox: {
    backgroundColor: 'rgba(239, 68, 68, 0.1)',
    border: '1px solid #ef4444',
    color: '#fca5a5',
    padding: '10px',
    borderRadius: '8px',
    marginBottom: '20px',
    fontSize: '13px',
  },
  // Active Session Card
  activeCard: {
    border: '1px solid #059669', // green-600
    padding: '30px',
    borderRadius: '16px',
    backgroundColor: 'rgba(5, 150, 105, 0.1)', // green tint
    textAlign: 'center',
    flexGrow: 1,
    display: 'flex',
    flexDirection: 'column',
    justifyContent: 'center',
    alignItems: 'center',
    boxShadow: '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)',
  },
  cardTitle: {
    marginTop: 0,
    fontSize: '18px',
    color: 'white',
    fontWeight: '600',
  },
  taskText: {
    color: '#d1d5db',
    fontSize: '14px',
    marginBottom: '20px',
  },
  timerDisplay: {
    fontSize: '56px',
    fontWeight: '700',
    margin: '20px 0',
    fontFamily: 'monospace',
    color: '#4ade80',
    letterSpacing: '0.05em',
  },
  stopButton: {
    backgroundColor: '#ef4444', // red-500
    color: 'white',
    padding: '12px 32px',
    border: 'none',
    borderRadius: '12px',
    fontSize: '16px',
    fontWeight: 'bold',
    cursor: 'pointer',
    transition: 'background-color 0.2s',
    boxShadow: '0 4px 6px -1px rgba(220, 38, 38, 0.3)',
  },
  // Inactive Session Card
  inactiveCard: {
    border: '1px solid #374151', // gray-700
    padding: '30px',
    borderRadius: '16px',
    backgroundColor: '#1f2937', // gray-800
    flexGrow: 1,
    display: 'flex',
    flexDirection: 'column',
    justifyContent: 'center',
  },
  label: {
    display: 'block',
    marginBottom: '8px',
    color: '#9ca3af', // gray-400
    fontSize: '13px',
    fontWeight: '500',
  },
  select: {
    width: '100%',
    padding: '12px',
    marginBottom: '0',
    borderRadius: '8px',
    border: '1px solid #4b5563', // gray-600
    backgroundColor: '#111827', // gray-900
    color: 'white',
    fontSize: '14px',
    outline: 'none',
  },
  startButton: {
    width: '100%',
    backgroundColor: '#22c55e', // green-500
    color: 'white',
    padding: '14px',
    border: 'none',
    borderRadius: '12px',
    fontSize: '16px',
    fontWeight: 'bold',
    marginTop: '20px',
    transition: 'all 0.2s',
    boxShadow: '0 4px 6px -1px rgba(34, 197, 94, 0.3)',
  }
};