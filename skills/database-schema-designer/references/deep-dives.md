
# Schema Design Deep Dives (Syntax Reference)

Syntax and worked examples for the decision calls made in `SKILL.md`. Load this file only after
a decision has been made and you need the concrete SQL/DDL to implement it — it is reference
material, not judgment.

---

## Data Types

### String Types

| Type | Use Case | Example |
|------|----------|---------|
| CHAR(n) | Fixed length | State codes, ISO dates |
| VARCHAR(n) | Variable length | Names, emails |
| TEXT | Long content | Articles, descriptions |

```sql
email VARCHAR(255)
phone VARCHAR(20)
country_code CHAR(2)
```

### Numeric Types

| Type | Range | Use Case |
|------|-------|----------|
| TINYINT | -128 to 127 | Age, status codes |
| SMALLINT | -32K to 32K | Quantities |
| INT | -2.1B to 2.1B | IDs, counts |
| BIGINT | Very large | Large IDs, timestamps |
| DECIMAL(p,s) | Exact precision | Money |
| FLOAT/DOUBLE | Approximate | Scientific data |

```sql
price DECIMAL(10, 2)  -- ALWAYS for money — FLOAT rounds
```

### Date/Time Types

```sql
DATE        -- 2025-10-31
TIME        -- 14:30:00
DATETIME    -- 2025-10-31 14:30:00
TIMESTAMP   -- Auto timezone conversion

created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
```

---

## Indexing Strategy

### When to Create Indexes

| Always Index | Reason |
|--------------|--------|
| Foreign keys | Speed up JOINs |
| WHERE clause columns | Speed up filtering |
| ORDER BY columns | Speed up sorting |
| Unique constraints | Enforced uniqueness |

```sql
CREATE INDEX idx_orders_customer ON orders(customer_id);
CREATE INDEX idx_orders_status_date ON orders(status, created_at);
```

### Index Types

| Type | Best For | Example |
|------|----------|---------|
| B-Tree | Ranges, equality | `price > 100` |
| Hash | Exact matches only | `email = 'x@y.com'` |
| Full-text | Text search | `MATCH AGAINST` |
| Partial | Subset of rows | `WHERE is_active = true` |
| GIN (Postgres) | JSONB/array containment | `data @> '{"k":"v"}'` |

### Composite Index Order

```sql
CREATE INDEX idx_customer_status ON orders(customer_id, status);

-- Uses index (customer_id first)
SELECT * FROM orders WHERE customer_id = 123;
SELECT * FROM orders WHERE customer_id = 123 AND status = 'pending';

-- Does NOT use index (status alone)
SELECT * FROM orders WHERE status = 'pending';
```

**Rule:** most selective column first, or the column most often queried alone.

---

## Constraints

```sql
-- Primary keys
id INT AUTO_INCREMENT PRIMARY KEY                    -- simple
id CHAR(36) PRIMARY KEY DEFAULT (UUID())             -- distributed systems
PRIMARY KEY (student_id, course_id)                  -- composite / junction table

-- Foreign keys
FOREIGN KEY (customer_id) REFERENCES customers(id)
  ON DELETE CASCADE     -- delete children with parent
  ON DELETE RESTRICT    -- prevent deletion if referenced
  ON DELETE SET NULL    -- set to NULL when parent deleted

-- Other
email VARCHAR(255) UNIQUE NOT NULL
UNIQUE (student_id, course_id)                       -- composite unique
price DECIMAL(10,2) CHECK (price >= 0)
```

---

## Relationship Patterns

```sql
-- One-to-many
CREATE TABLE order_items (
  id INT PRIMARY KEY,
  order_id INT NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
  product_id INT NOT NULL,
  quantity INT NOT NULL
);

-- Many-to-many (junction table)
CREATE TABLE enrollments (
  student_id INT REFERENCES students(id) ON DELETE CASCADE,
  course_id INT REFERENCES courses(id) ON DELETE CASCADE,
  enrolled_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (student_id, course_id)
);

-- Self-referencing
CREATE TABLE employees (
  id INT PRIMARY KEY,
  name VARCHAR(100) NOT NULL,
  manager_id INT REFERENCES employees(id)
);

-- Polymorphic — Approach 1: separate FKs (stronger integrity)
CREATE TABLE comments (
  id INT PRIMARY KEY,
  content TEXT NOT NULL,
  post_id INT REFERENCES posts(id),
  photo_id INT REFERENCES photos(id),
  CHECK (
    (post_id IS NOT NULL AND photo_id IS NULL) OR
    (post_id IS NULL AND photo_id IS NOT NULL)
  )
);

-- Polymorphic — Approach 2: type + id (flexible, weaker integrity, no DB-level FK)
CREATE TABLE comments (
  id INT PRIMARY KEY,
  content TEXT NOT NULL,
  commentable_type VARCHAR(50) NOT NULL,
  commentable_id INT NOT NULL
);
```

---

## NoSQL Design (MongoDB)

```json
// Embedded — read together, 1:few, small doc, rarely updated independently
{
  "_id": "order_123",
  "customer": { "id": "cust_456", "name": "Jane Smith" },
  "items": [{ "product_id": "prod_789", "quantity": 2, "price": 29.99 }],
  "total": 109.97
}

// Referenced — read separately, 1:many, approaching 16MB doc limit, frequent updates
{ "_id": "order_123", "customer_id": "cust_456", "item_ids": ["item_1", "item_2"], "total": 109.97 }
```

```javascript
db.users.createIndex({ email: 1 }, { unique: true });
db.orders.createIndex({ customer_id: 1, created_at: -1 });
db.articles.createIndex({ title: "text", content: "text" });
db.stores.createIndex({ location: "2dsphere" });
```

---

## Migrations

### Adding a Column (Zero-Downtime)

```sql
-- Step 1: add nullable column
ALTER TABLE users ADD COLUMN phone VARCHAR(20);
-- Step 2: deploy code that writes to new column
-- Step 3: backfill existing rows
UPDATE users SET phone = '' WHERE phone IS NULL;
-- Step 4: make required (if needed)
ALTER TABLE users MODIFY phone VARCHAR(20) NOT NULL;
```

### Renaming a Column (Zero-Downtime)

```sql
-- Step 1: add new column, Step 2: copy data
ALTER TABLE users ADD COLUMN email_address VARCHAR(255);
UPDATE users SET email_address = email;
-- Step 3/4: deploy code reading then writing the new column
-- Step 5: drop old column once nothing reads it
ALTER TABLE users DROP COLUMN email;
```

See `assets/templates/migration-template.sql` for the up/down transaction skeleton.

---

## Performance Optimization

```sql
EXPLAIN SELECT * FROM orders WHERE customer_id = 123 AND status = 'pending';
```

| Look For | Meaning |
|----------|---------|
| type: ALL | Full table scan (bad) |
| type: ref | Index used (good) |
| key: NULL | No index used |
| rows: high | Many rows scanned |

```python
# BAD: N+1 queries
orders = db.query("SELECT * FROM orders")
for order in orders:
    customer = db.query(f"SELECT * FROM customers WHERE id = {order.customer_id}")

# GOOD: single JOIN
results = db.query("""
    SELECT orders.*, customers.name FROM orders
    JOIN customers ON orders.customer_id = customers.id
""")
```

| Technique | When to Use |
|-----------|-------------|
| Add indexes | Slow WHERE/ORDER BY |
| Denormalize | Expensive JOINs |
| Pagination | Large result sets |
| Caching | Repeated queries |
| Read replicas | Read-heavy load |
| Partitioning | Very large tables |
