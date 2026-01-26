import React from 'react';
import { useTranslation } from 'react-i18next';
import type { RepeatingDecimalFormats } from '../types';

interface Props {
  formats: RepeatingDecimalFormats;
}

/**
 * RepeatingDecimalNotations displays a table of different notation formats
 * for repeating decimals as described in the Wikipedia article on repeating decimals.
 *
 * @see https://en.wikipedia.org/wiki/Repeating_decimal
 */
const RepeatingDecimalNotations: React.FC<Props> = ({ formats }) => {
  const { t } = useTranslation();

  const notations = [
    { name: t('notations.vinculum', 'Vinculum (overline)'), value: formats.vinculum },
    { name: t('notations.parenthesis', 'Parenthesis'), value: formats.parenthesis },
    { name: t('notations.ellipsis', 'Ellipsis'), value: formats.ellipsis },
    { name: t('notations.fraction', 'Fraction'), value: formats.fraction },
  ];

  return (
    <div className="repeating-decimal-notations">
      <table className="notations-table">
        <thead>
          <tr>
            <th>{t('notations.notation', 'Notation')}</th>
            <th>{t('notations.representation', 'Representation')}</th>
          </tr>
        </thead>
        <tbody>
          {notations.map((n, idx) => (
            <tr key={idx}>
              <td>{n.name}</td>
              <td className="notation-value">{n.value}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
};

export default RepeatingDecimalNotations;
